//! Builder for creating IoEngine instances.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::{
    FileExistsPolicy, InputConfig, InputSpec, OutputConfig, OutputSpec, PipelineConfig,
};
use crate::engine::IoEngine;
use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
use crate::format::{CustomFormat, FormatKind, FormatRegistry};
use crate::io::{FileInput, FileOutput, InputProvider, OutputTarget, StdinInput, StdoutOutput};

pub struct MultiioBuilder {
    input_args: Vec<String>,
    output_args: Vec<String>,
    input_specs: Vec<InputSpec>,
    output_specs: Vec<OutputSpec>,
    registry: FormatRegistry,
    error_policy: ErrorPolicy,
    default_input_formats: Vec<FormatKind>,
    default_output_formats: Vec<FormatKind>,
    file_exists_policy: FileExistsPolicy,
}

impl MultiioBuilder {
    pub fn new(registry: FormatRegistry) -> Self {
        Self {
            input_args: Vec::new(),
            output_args: Vec::new(),
            input_specs: Vec::new(),
            output_specs: Vec::new(),
            registry,
            error_policy: ErrorPolicy::Accumulate,
            default_input_formats: vec![FormatKind::Json, FormatKind::Yaml, FormatKind::Plaintext],
            default_output_formats: vec![FormatKind::Json, FormatKind::Yaml, FormatKind::Plaintext],
            file_exists_policy: FileExistsPolicy::Overwrite,
        }
    }

    pub fn with_custom_format(mut self, format: CustomFormat) -> Self {
        self.registry.register_custom(format);
        self
    }

    pub fn inputs_from_args(mut self, args: &[String]) -> Self {
        self.input_args = args.to_vec();
        self
    }

    pub fn outputs_from_args(mut self, args: &[String]) -> Self {
        self.output_args = args.to_vec();
        self
    }

    pub fn add_input(mut self, arg: impl Into<String>) -> Self {
        self.input_args.push(arg.into());
        self
    }

    pub fn add_output(mut self, arg: impl Into<String>) -> Self {
        self.output_args.push(arg.into());
        self
    }

    pub fn add_input_spec(mut self, spec: InputSpec) -> Self {
        self.input_specs.push(spec);
        self
    }

    pub fn add_output_spec(mut self, spec: OutputSpec) -> Self {
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

    pub fn build(self) -> Result<IoEngine, AggregateError> {
        let mut inputs = self.resolve_inputs()?;
        let mut outputs = self.resolve_outputs()?;

        // Add pre-built specs
        inputs.extend(self.input_specs);
        outputs.extend(self.output_specs);

        Ok(IoEngine::new(
            self.registry,
            self.error_policy,
            inputs,
            outputs,
        ))
    }

    fn resolve_inputs(&self) -> Result<Vec<InputSpec>, AggregateError> {
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

    fn resolve_single_input(&self, raw: &str) -> Result<InputSpec, SingleIoError> {
        if raw == "-" {
            return Ok(InputSpec {
                raw: raw.to_string(),
                provider: Arc::new(StdinInput::new()),
                explicit_format: None,
                format_candidates: self.default_input_formats.clone(),
            });
        }

        let path = PathBuf::from(raw);
        let provider: Arc<dyn InputProvider> = Arc::new(FileInput::new(path.clone()));

        let explicit = self.infer_format_from_path(&path);

        Ok(InputSpec {
            raw: raw.to_string(),
            provider,
            explicit_format: explicit,
            format_candidates: self.default_input_formats.clone(),
        })
    }

    fn resolve_outputs(&self) -> Result<Vec<OutputSpec>, AggregateError> {
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

    fn resolve_single_output(&self, raw: &str) -> Result<OutputSpec, SingleIoError> {
        if raw == "-" {
            return Ok(OutputSpec {
                raw: raw.to_string(),
                target: Arc::new(StdoutOutput::new()),
                explicit_format: None,
                format_candidates: self.default_output_formats.clone(),
                file_exists_policy: self.file_exists_policy,
            });
        }

        let path = PathBuf::from(raw);
        let target: Arc<dyn OutputTarget> = Arc::new(FileOutput::new(path.clone()));

        let explicit = self.infer_format_from_path(&path);

        Ok(OutputSpec {
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
        registry: FormatRegistry,
    ) -> Result<Self, AggregateError> {
        let mut builder = MultiioBuilder::new(registry);

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

        let mut errors = Vec::with_capacity(config.inputs.len() + config.outputs.len());
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

    fn input_from_config(&self, cfg: &InputConfig) -> Result<InputSpec, SingleIoError> {
        let provider: Arc<dyn InputProvider> = match cfg.kind.as_str() {
            "stdin" | "-" => Arc::new(StdinInput::new()),
            "file" => {
                let path = cfg.path.as_ref().ok_or_else(|| SingleIoError {
                    stage: Stage::ResolveInput,
                    target: cfg.id.clone(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "file input requires 'path' field",
                    )),
                })?;
                Arc::new(FileInput::new(PathBuf::from(path)))
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

        Ok(InputSpec {
            raw: cfg.id.clone(),
            provider,
            explicit_format,
            format_candidates: self.default_input_formats.clone(),
        })
    }

    fn output_from_config(&self, cfg: &OutputConfig) -> Result<OutputSpec, SingleIoError> {
        let target: Arc<dyn OutputTarget> = match cfg.kind.as_str() {
            "stdout" | "-" => Arc::new(StdoutOutput::new()),
            "stderr" => Arc::new(crate::io::StderrOutput::new()),
            "file" => {
                let path = cfg.path.as_ref().ok_or_else(|| SingleIoError {
                    stage: Stage::ResolveOutput,
                    target: cfg.id.clone(),
                    error: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "file output requires 'path' field",
                    )),
                })?;
                Arc::new(FileOutput::new(PathBuf::from(path)))
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

        Ok(OutputSpec {
            raw: cfg.id.clone(),
            target,
            explicit_format,
            format_candidates: self.default_output_formats.clone(),
            file_exists_policy,
        })
    }
}

impl Default for MultiioBuilder {
    fn default() -> Self {
        MultiioBuilder::new(crate::format::default_registry())
    }
}
