//! Async builder for creating AsyncIoEngine instances.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::{
    AsyncInputSpec, AsyncOutputSpec, FileExistsPolicy, InputConfig, OutputConfig, PipelineConfig,
};
use crate::engine_async::AsyncIoEngine;
use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
#[cfg(feature = "custom")]
use crate::format::CustomFormat;
use crate::format::{
    AsyncFormatRegistry, DEFAULT_FORMAT_ORDER, FormatKind, FormatRegistry, default_async_registry,
    default_registry,
};
use crate::io::{
    AsyncFileInput, AsyncFileOutput, AsyncInMemorySource, AsyncInputProvider, AsyncOutputTarget,
    AsyncStderrOutput, AsyncStdinInput, AsyncStdoutOutput,
};

pub struct MultiioAsyncBuilder {
    input_args: Vec<String>,
    output_args: Vec<String>,
    input_specs: Vec<AsyncInputSpec>,
    output_specs: Vec<AsyncOutputSpec>,
    registry: AsyncFormatRegistry,
    sync_registry: FormatRegistry,
    error_policy: ErrorPolicy,
    default_input_formats: Vec<FormatKind>,
    default_output_formats: Vec<FormatKind>,
    file_exists_policy: FileExistsPolicy,
}

impl Default for MultiioAsyncBuilder {
    fn default() -> Self {
        MultiioAsyncBuilder::new(default_async_registry())
    }
}

impl MultiioAsyncBuilder {
    pub fn new(registry: AsyncFormatRegistry) -> Self {
        Self {
            input_args: Vec::new(),
            output_args: Vec::new(),
            input_specs: Vec::new(),
            output_specs: Vec::new(),
            registry,
            sync_registry: default_registry(),
            error_policy: ErrorPolicy::Accumulate,
            default_input_formats: DEFAULT_FORMAT_ORDER.to_vec(),
            default_output_formats: DEFAULT_FORMAT_ORDER.to_vec(),
            file_exists_policy: FileExistsPolicy::Overwrite,
        }
    }

    /// Replace the underlying sync `FormatRegistry` used for decoding,
    /// encoding, and streaming.
    pub fn with_sync_registry(mut self, registry: FormatRegistry) -> Self {
        self.sync_registry = registry;
        self
    }

    /// Register a custom format on the sync registry and mark it available on
    /// the async registry.
    ///
    /// This ensures that custom formats participate fully in decode/encode and
    /// streaming via the sync `FormatRegistry`, while the async registry is
    /// aware of the corresponding `FormatKind::Custom` for feature gating or
    /// extension-based resolution paths.
    #[cfg(feature = "custom")]
    pub fn with_custom_format(mut self, format: CustomFormat) -> Self {
        // Register the custom format in the sync registry so that
        // `deserialize_value`, `serialize_value`, and
        // `stream_deserialize_into` can use it.
        self.sync_registry.register_custom(format.clone());

        // Also mark the custom kind as available in the async registry.
        self.registry.register(FormatKind::Custom(format.name));

        self
    }

    pub fn inputs_from_args(mut self, args: &[String]) -> Self {
        self.input_args = args.to_vec();
        self
    }

    pub fn with_input_args(self, args: &crate::cli::InputArgs) -> Self {
        self.inputs_from_args(args.as_slice())
    }

    pub fn outputs_from_args(mut self, args: &[String]) -> Self {
        self.output_args = args.to_vec();
        self
    }

    pub fn with_output_args(self, args: &crate::cli::OutputArgs) -> Self {
        self.outputs_from_args(args.as_slice())
    }

    pub fn add_input(mut self, arg: impl Into<String>) -> Self {
        self.input_args.push(arg.into());
        self
    }

    pub fn add_output(mut self, arg: impl Into<String>) -> Self {
        self.output_args.push(arg.into());
        self
    }

    pub fn add_input_spec(mut self, spec: AsyncInputSpec) -> Self {
        self.input_specs.push(spec);
        self
    }

    pub fn add_output_spec(mut self, spec: AsyncOutputSpec) -> Self {
        self.output_specs.push(spec);
        self
    }

    pub fn with_order(mut self, order: &[FormatKind]) -> Self {
        self.default_input_formats = order.to_vec();
        self.default_output_formats = order.to_vec();
        self
    }

    pub fn with_input_order(mut self, order: &[FormatKind]) -> Self {
        self.default_input_formats = order.to_vec();
        self
    }

    pub fn with_output_order(mut self, order: &[FormatKind]) -> Self {
        self.default_output_formats = order.to_vec();
        self
    }

    pub fn with_mode(mut self, policy: ErrorPolicy) -> Self {
        self.error_policy = policy;
        self
    }

    pub fn with_file_exists_policy(mut self, policy: FileExistsPolicy) -> Self {
        self.file_exists_policy = policy;
        self
    }

    pub fn build(self) -> Result<AsyncIoEngine, AggregateError> {
        let mut inputs = self.resolve_inputs()?;
        let mut outputs = self.resolve_outputs()?;

        // Add pre-built specs
        inputs.extend(self.input_specs);
        outputs.extend(self.output_specs);

        Ok(AsyncIoEngine::new_with_sync_registry(
            self.registry,
            self.sync_registry,
            self.error_policy,
            inputs,
            outputs,
        ))
    }

    fn resolve_inputs(&self) -> Result<Vec<AsyncInputSpec>, AggregateError> {
        let mut specs = Vec::with_capacity(self.input_args.len());
        let mut errors = Vec::new();

        for raw in &self.input_args {
            match self.resolve_single_input(raw) {
                Ok(spec) => specs.push(spec),
                Err(e) => {
                    errors.push(e);
                    if matches!(self.error_policy, ErrorPolicy::FastFail) {
                        return Err(AggregateError { errors });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(specs)
        } else {
            Err(AggregateError { errors })
        }
    }

    fn resolve_single_input(&self, raw: &str) -> Result<AsyncInputSpec, SingleIoError> {
        let raw = raw.trim();

        if let Some(path) = raw.strip_prefix('@') {
            if path.is_empty() {
                return Err(SingleIoError {
                    stage: Stage::ResolveInput,
                    target: raw.to_string(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "expected a path after '@'",
                    )),
                });
            }

            let path = PathBuf::from(path);
            let provider: Arc<dyn AsyncInputProvider> = Arc::new(AsyncFileInput::new(path.clone()));
            let explicit = self.infer_format_from_path(&path);

            return Ok(AsyncInputSpec {
                raw: path.to_string_lossy().into_owned(),
                provider,
                explicit_format: explicit,
                format_candidates: self.default_input_formats.clone(),
            });
        }

        if raw == "-" || raw.eq_ignore_ascii_case("stdin") {
            return Ok(AsyncInputSpec {
                raw: "-".to_string(),
                provider: Arc::new(AsyncStdinInput::new()),
                explicit_format: None,
                format_candidates: self.default_input_formats.clone(),
            });
        }

        if let Some(content) = raw.strip_prefix('=') {
            use std::hash::{Hash, Hasher};

            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            content.hash(&mut hasher);
            let id = format!("inline:{:016x}", hasher.finish());

            let provider: Arc<dyn AsyncInputProvider> = Arc::new(AsyncInMemorySource::from_string(
                id.clone(),
                content.to_string(),
            ));

            return Ok(AsyncInputSpec {
                raw: id,
                provider,
                explicit_format: None,
                format_candidates: self.default_input_formats.clone(),
            });
        }

        let path = PathBuf::from(raw);
        let provider: Arc<dyn AsyncInputProvider> = Arc::new(AsyncFileInput::new(path.clone()));
        let explicit = self.infer_format_from_path(&path);

        Ok(AsyncInputSpec {
            raw: raw.to_string(),
            provider,
            explicit_format: explicit,
            format_candidates: self.default_input_formats.clone(),
        })
    }

    fn resolve_outputs(&self) -> Result<Vec<AsyncOutputSpec>, AggregateError> {
        let mut specs = Vec::with_capacity(self.output_args.len());
        let mut errors = Vec::new();

        for raw in &self.output_args {
            match self.resolve_single_output(raw) {
                Ok(spec) => specs.push(spec),
                Err(e) => {
                    errors.push(e);
                    if matches!(self.error_policy, ErrorPolicy::FastFail) {
                        return Err(AggregateError { errors });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(specs)
        } else {
            Err(AggregateError { errors })
        }
    }

    fn resolve_single_output(&self, raw: &str) -> Result<AsyncOutputSpec, SingleIoError> {
        let raw = raw.trim();

        if let Some(path) = raw.strip_prefix('@') {
            if path.is_empty() {
                return Err(SingleIoError {
                    stage: Stage::ResolveOutput,
                    target: raw.to_string(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "expected a path after '@'",
                    )),
                });
            }

            let path = PathBuf::from(path);
            let target: Arc<dyn AsyncOutputTarget> = Arc::new(AsyncFileOutput::new(path.clone()));
            let explicit = self.infer_format_from_path(&path);

            return Ok(AsyncOutputSpec {
                raw: path.to_string_lossy().into_owned(),
                target,
                explicit_format: explicit,
                format_candidates: self.default_output_formats.clone(),
                file_exists_policy: self.file_exists_policy,
            });
        }

        if raw == "-" || raw.eq_ignore_ascii_case("stdout") {
            return Ok(AsyncOutputSpec {
                raw: "-".to_string(),
                target: Arc::new(AsyncStdoutOutput::new()),
                explicit_format: None,
                format_candidates: self.default_output_formats.clone(),
                file_exists_policy: self.file_exists_policy,
            });
        }

        if raw.eq_ignore_ascii_case("stderr") {
            return Ok(AsyncOutputSpec {
                raw: "stderr".to_string(),
                target: Arc::new(AsyncStderrOutput::new()),
                explicit_format: None,
                format_candidates: self.default_output_formats.clone(),
                file_exists_policy: self.file_exists_policy,
            });
        }

        let path = PathBuf::from(raw);
        let target: Arc<dyn AsyncOutputTarget> = Arc::new(AsyncFileOutput::new(path.clone()));

        let explicit = self.infer_format_from_path(&path);

        Ok(AsyncOutputSpec {
            raw: raw.to_string(),
            target,
            explicit_format: explicit,
            format_candidates: self.default_output_formats.clone(),
            file_exists_policy: self.file_exists_policy,
        })
    }

    fn infer_format_from_path(&self, path: &Path) -> Option<FormatKind> {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
            .and_then(|ext| self.registry.kind_for_extension(ext))
    }

    pub fn from_pipeline_config(
        config: PipelineConfig,
        registry: AsyncFormatRegistry,
    ) -> Result<Self, AggregateError> {
        let mut builder = MultiioAsyncBuilder::new(registry);

        if let Some(policy_str) = config.error_policy.as_deref() {
            let policy = match policy_str {
                "fast_fail" | "fastfail" => ErrorPolicy::FastFail,
                _ => ErrorPolicy::Accumulate,
            };
            builder = builder.with_mode(policy);
        }

        if let Some(order) = config.format_order.as_ref() {
            let kinds: Vec<FormatKind> = order
                .iter()
                .filter_map(|s| s.parse::<FormatKind>().ok())
                .collect();
            builder = builder.with_order(&kinds);
        }

        let mut errors = Vec::new();

        for input_cfg in config.inputs {
            match builder.input_from_config(&input_cfg) {
                Ok(spec) => builder.input_specs.push(spec),
                Err(e) => {
                    errors.push(e);
                    if matches!(builder.error_policy, ErrorPolicy::FastFail) {
                        return Err(AggregateError { errors });
                    }
                }
            }
        }

        for output_cfg in config.outputs {
            match builder.output_from_config(&output_cfg) {
                Ok(spec) => builder.output_specs.push(spec),
                Err(e) => {
                    errors.push(e);
                    if matches!(builder.error_policy, ErrorPolicy::FastFail) {
                        return Err(AggregateError { errors });
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(AggregateError { errors });
        }

        Ok(builder)
    }

    fn input_from_config(&self, cfg: &InputConfig) -> Result<AsyncInputSpec, SingleIoError> {
        let provider: Arc<dyn AsyncInputProvider> = match cfg.kind.as_str() {
            "stdin" | "-" => Arc::new(AsyncStdinInput::new()),
            "file" => {
                let path = cfg.path.as_ref().ok_or_else(|| SingleIoError {
                    stage: Stage::ResolveInput,
                    target: cfg.id.clone(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "file input requires 'path' field",
                    )),
                })?;
                Arc::new(AsyncFileInput::new(PathBuf::from(path)))
            }
            other => {
                return Err(SingleIoError {
                    stage: Stage::ResolveInput,
                    target: cfg.id.clone(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("unknown input kind: {}", other),
                    )),
                });
            }
        };

        let explicit_format = cfg
            .format
            .as_ref()
            .and_then(|s| s.parse::<FormatKind>().ok());

        Ok(AsyncInputSpec {
            raw: cfg.id.clone(),
            provider,
            explicit_format,
            format_candidates: self.default_input_formats.clone(),
        })
    }

    fn output_from_config(&self, cfg: &OutputConfig) -> Result<AsyncOutputSpec, SingleIoError> {
        let target: Arc<dyn AsyncOutputTarget> = match cfg.kind.as_str() {
            "stdout" | "-" => Arc::new(AsyncStdoutOutput::new()),
            "stderr" => Arc::new(AsyncStderrOutput::new()),
            "file" => {
                let path = cfg.path.as_ref().ok_or_else(|| SingleIoError {
                    stage: Stage::ResolveOutput,
                    target: cfg.id.clone(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "file output requires 'path' field",
                    )),
                })?;
                Arc::new(AsyncFileOutput::new(PathBuf::from(path)))
            }
            other => {
                return Err(SingleIoError {
                    stage: Stage::ResolveOutput,
                    target: cfg.id.clone(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("unknown output kind: {}", other),
                    )),
                });
            }
        };

        let explicit_format = cfg
            .format
            .as_ref()
            .and_then(|s| s.parse::<FormatKind>().ok());

        let file_exists_policy = cfg
            .file_exists_policy
            .as_ref()
            .and_then(|s| s.parse::<FileExistsPolicy>().ok())
            .unwrap_or(self.file_exists_policy);

        Ok(AsyncOutputSpec {
            raw: cfg.id.clone(),
            target,
            explicit_format,
            format_candidates: self.default_output_formats.clone(),
            file_exists_policy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{DEFAULT_FORMAT_ORDER, FormatKind, default_async_registry};

    #[test]
    fn async_builder_defaults_match_default_format_order() {
        let builder = MultiioAsyncBuilder::new(default_async_registry());
        assert_eq!(builder.default_input_formats, DEFAULT_FORMAT_ORDER);
        assert_eq!(builder.default_output_formats, DEFAULT_FORMAT_ORDER);
    }

    #[test]
    fn resolve_single_input_supports_stdin_alias_and_inline_content() {
        let builder = MultiioAsyncBuilder::default();

        let stdin = builder.resolve_single_input("stdin").expect("stdin spec");
        assert_eq!(stdin.raw, "-");
        assert_eq!(stdin.provider.id(), "-");
        assert!(stdin.explicit_format.is_none());

        let inline = builder.resolve_single_input("=hello").expect("inline spec");
        assert!(inline.raw.starts_with("inline:"));
        assert_eq!(inline.provider.id(), inline.raw);
        assert!(inline.explicit_format.is_none());

        let forced_path = builder
            .resolve_single_input("@file.txt")
            .expect("forced path spec");
        assert_eq!(forced_path.raw, "file.txt");
        assert_eq!(forced_path.provider.id(), "file.txt");
        assert_eq!(forced_path.explicit_format, Some(FormatKind::Plaintext));
    }

    #[test]
    fn resolve_single_output_supports_stdout_alias_stderr_and_forced_path() {
        let builder = MultiioAsyncBuilder::default();

        let stdout = builder
            .resolve_single_output("stdout")
            .expect("stdout spec");
        assert_eq!(stdout.raw, "-");
        assert_eq!(stdout.target.id(), "-");
        assert!(stdout.explicit_format.is_none());

        let stderr = builder
            .resolve_single_output("stderr")
            .expect("stderr spec");
        assert_eq!(stderr.raw, "stderr");
        assert_eq!(stderr.target.id(), "stderr");
        assert!(stderr.explicit_format.is_none());

        let forced_path = builder
            .resolve_single_output("@out.txt")
            .expect("forced path spec");
        assert_eq!(forced_path.raw, "out.txt");
        assert_eq!(forced_path.target.id(), "out.txt");
        assert_eq!(forced_path.explicit_format, Some(FormatKind::Plaintext));
    }
}
