//! Async builder for creating AsyncIoEngine instances.

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::{AsyncInputSpec, AsyncOutputSpec, FileExistsPolicy, PipelineConfig};
use crate::engine_async::AsyncIoEngine;
use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
use crate::format::{AsyncFormatRegistry, FormatKind};
use crate::io::{
    AsyncFileInput, AsyncFileOutput, AsyncInputProvider, AsyncOutputTarget, AsyncStdinInput,
    AsyncStdoutOutput,
};

/// Builder for creating AsyncIoEngine instances.
pub struct MultiioAsyncBuilder {
    input_args: Vec<String>,
    output_args: Vec<String>,
    input_specs: Vec<AsyncInputSpec>,
    output_specs: Vec<AsyncOutputSpec>,
    registry: AsyncFormatRegistry,
    error_policy: ErrorPolicy,
    default_input_formats: Vec<FormatKind>,
    default_output_formats: Vec<FormatKind>,
    file_exists_policy: FileExistsPolicy,
}

impl MultiioAsyncBuilder {
    /// Create a new async builder with the given format registry.
    pub fn new(registry: AsyncFormatRegistry) -> Self {
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

    /// Set input arguments from command line args.
    pub fn inputs_from_args(mut self, args: &[String]) -> Self {
        self.input_args = args.to_vec();
        self
    }

    /// Set output arguments from command line args.
    pub fn outputs_from_args(mut self, args: &[String]) -> Self {
        self.output_args = args.to_vec();
        self
    }

    /// Add a single input argument.
    pub fn add_input(mut self, arg: impl Into<String>) -> Self {
        self.input_args.push(arg.into());
        self
    }

    /// Add a single output argument.
    pub fn add_output(mut self, arg: impl Into<String>) -> Self {
        self.output_args.push(arg.into());
        self
    }

    /// Add a pre-built async input specification.
    pub fn add_input_spec(mut self, spec: AsyncInputSpec) -> Self {
        self.input_specs.push(spec);
        self
    }

    /// Add a pre-built async output specification.
    pub fn add_output_spec(mut self, spec: AsyncOutputSpec) -> Self {
        self.output_specs.push(spec);
        self
    }

    /// Set the format priority order for both inputs and outputs.
    pub fn with_order(mut self, order: &[FormatKind]) -> Self {
        self.default_input_formats = order.to_vec();
        self.default_output_formats = order.to_vec();
        self
    }

    /// Set the error handling policy.
    pub fn with_mode(mut self, policy: ErrorPolicy) -> Self {
        self.error_policy = policy;
        self
    }

    /// Set the file exists policy for outputs.
    pub fn with_file_exists_policy(mut self, policy: FileExistsPolicy) -> Self {
        self.file_exists_policy = policy;
        self
    }

    /// Build the AsyncIoEngine from the current configuration.
    pub fn build(self) -> Result<AsyncIoEngine, AggregateError> {
        let mut inputs = self.resolve_inputs()?;
        let mut outputs = self.resolve_outputs()?;

        // Add pre-built specs
        inputs.extend(self.input_specs);
        outputs.extend(self.output_specs);

        Ok(AsyncIoEngine::new(
            self.registry,
            self.error_policy,
            inputs,
            outputs,
        ))
    }

    /// Resolve input arguments into AsyncInputSpecs.
    fn resolve_inputs(&self) -> Result<Vec<AsyncInputSpec>, AggregateError> {
        let mut specs = Vec::new();
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

    /// Resolve a single input argument into an AsyncInputSpec.
    fn resolve_single_input(&self, raw: &str) -> Result<AsyncInputSpec, SingleIoError> {
        if raw == "-" {
            return Ok(AsyncInputSpec {
                raw: raw.to_string(),
                provider: Arc::new(AsyncStdinInput::new()),
                explicit_format: None,
                format_candidates: self.default_input_formats.clone(),
            });
        }

        let path = PathBuf::from(raw);
        let provider: Arc<dyn AsyncInputProvider> = Arc::new(AsyncFileInput::new(path.clone()));

        // Try to detect format from extension
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        let explicit = ext
            .as_deref()
            .and_then(|e| self.registry.kind_for_extension(e));

        Ok(AsyncInputSpec {
            raw: raw.to_string(),
            provider,
            explicit_format: explicit,
            format_candidates: self.default_input_formats.clone(),
        })
    }

    /// Resolve output arguments into AsyncOutputSpecs.
    fn resolve_outputs(&self) -> Result<Vec<AsyncOutputSpec>, AggregateError> {
        let mut specs = Vec::new();
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

    /// Resolve a single output argument into an AsyncOutputSpec.
    fn resolve_single_output(&self, raw: &str) -> Result<AsyncOutputSpec, SingleIoError> {
        if raw == "-" {
            return Ok(AsyncOutputSpec {
                raw: raw.to_string(),
                target: Arc::new(AsyncStdoutOutput::new()),
                explicit_format: None,
                format_candidates: self.default_output_formats.clone(),
                file_exists_policy: self.file_exists_policy,
            });
        }

        let path = PathBuf::from(raw);
        let target: Arc<dyn AsyncOutputTarget> = Arc::new(AsyncFileOutput::new(path.clone()));

        // Try to detect format from extension
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        let explicit = ext
            .as_deref()
            .and_then(|e| self.registry.kind_for_extension(e));

        Ok(AsyncOutputSpec {
            raw: raw.to_string(),
            target,
            explicit_format: explicit,
            format_candidates: self.default_output_formats.clone(),
            file_exists_policy: self.file_exists_policy,
        })
    }

    /// Create a builder from a pipeline configuration.
    pub fn from_pipeline_config(
        config: PipelineConfig,
        registry: AsyncFormatRegistry,
    ) -> Result<Self, AggregateError> {
        let mut builder = MultiioAsyncBuilder::new(registry);

        // Set error policy
        if let Some(policy_str) = config.error_policy.as_deref() {
            let policy = match policy_str {
                "fast_fail" | "fastfail" => ErrorPolicy::FastFail,
                _ => ErrorPolicy::Accumulate,
            };
            builder = builder.with_mode(policy);
        }

        // Set format order
        if let Some(order) = config.format_order.as_ref() {
            let kinds: Vec<FormatKind> = order
                .iter()
                .filter_map(|s| FormatKind::from_str(s))
                .collect();
            builder = builder.with_order(&kinds);
        }

        Ok(builder)
    }
}
