//! Pipeline configuration for defining I/O workflows.

use serde::Deserialize;

/// Configuration for an entire I/O pipeline.
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineConfig {
    /// Input configurations
    #[serde(default)]
    pub inputs: Vec<InputConfig>,
    /// Output configurations
    #[serde(default)]
    pub outputs: Vec<OutputConfig>,
    /// Error policy: "fast_fail" or "accumulate"
    #[serde(default)]
    pub error_policy: Option<String>,
    /// Format priority order
    #[serde(default)]
    pub format_order: Option<Vec<String>>,
}

/// Configuration for a single input source.
#[derive(Debug, Clone, Deserialize)]
pub struct InputConfig {
    /// Unique identifier for this input
    pub id: String,
    /// Kind of input: "stdin", "file", "http", etc.
    pub kind: String,
    /// File path (for file inputs)
    #[serde(default)]
    pub path: Option<String>,
    /// URL (for HTTP/network inputs)
    #[serde(default)]
    pub url: Option<String>,
    /// Explicit format: "json", "yaml", etc.
    #[serde(default)]
    pub format: Option<String>,
}

/// Configuration for a single output target.
#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    /// Unique identifier for this output
    pub id: String,
    /// Kind of output: "stdout", "stderr", "file", etc.
    pub kind: String,
    /// File path (for file outputs)
    #[serde(default)]
    pub path: Option<String>,
    /// Explicit format: "json", "yaml", etc.
    #[serde(default)]
    pub format: Option<String>,
    /// File exists policy: "overwrite", "append", "error"
    #[serde(default)]
    pub file_exists_policy: Option<String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            error_policy: None,
            format_order: None,
        }
    }
}

impl PipelineConfig {
    /// Create a new empty pipeline configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input configuration.
    pub fn add_input(mut self, input: InputConfig) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add an output configuration.
    pub fn add_output(mut self, output: OutputConfig) -> Self {
        self.outputs.push(output);
        self
    }

    /// Set the error policy.
    pub fn with_error_policy(mut self, policy: impl Into<String>) -> Self {
        self.error_policy = Some(policy.into());
        self
    }

    /// Set the format order.
    pub fn with_format_order(mut self, order: Vec<String>) -> Self {
        self.format_order = Some(order);
        self
    }
}
