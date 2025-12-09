//! Input and output specifications.

use std::sync::Arc;

use crate::format::FormatKind;
use crate::io::{InputProvider, OutputTarget};

/// Policy for handling existing output files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileExistsPolicy {
    /// Overwrite existing files
    Overwrite,
    /// Append to existing files
    Append,
    #[default]
    /// Return an error if file exists
    Error,
}

impl FileExistsPolicy {
    /// Parse a policy from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "overwrite" => Some(FileExistsPolicy::Overwrite),
            "append" => Some(FileExistsPolicy::Append),
            "error" => Some(FileExistsPolicy::Error),
            _ => None,
        }
    }
}

/// Specification for a single input source.
#[derive(Debug, Clone)]
pub struct InputSpec {
    /// Raw input argument or configuration string
    pub raw: String,
    /// The input provider implementation
    pub provider: Arc<dyn InputProvider>,
    /// Explicitly specified format (if any)
    pub explicit_format: Option<FormatKind>,
    /// Candidate formats to try (in order)
    pub format_candidates: Vec<FormatKind>,
}

impl InputSpec {
    /// Create a new input specification.
    pub fn new(raw: impl Into<String>, provider: Arc<dyn InputProvider>) -> Self {
        Self {
            raw: raw.into(),
            provider,
            explicit_format: None,
            format_candidates: Vec::new(),
        }
    }

    /// Set the explicit format.
    pub fn with_format(mut self, format: FormatKind) -> Self {
        self.explicit_format = Some(format);
        self
    }

    /// Set the format candidates.
    pub fn with_candidates(mut self, candidates: Vec<FormatKind>) -> Self {
        self.format_candidates = candidates;
        self
    }
}

/// Specification for a single output target.
#[derive(Debug, Clone)]
pub struct OutputSpec {
    /// Raw output argument or configuration string
    pub raw: String,
    /// The output target implementation
    pub target: Arc<dyn OutputTarget>,
    /// Explicitly specified format (if any)
    pub explicit_format: Option<FormatKind>,
    /// Candidate formats to try (in order)
    pub format_candidates: Vec<FormatKind>,
    /// Policy for handling existing files
    pub file_exists_policy: FileExistsPolicy,
}

impl OutputSpec {
    /// Create a new output specification.
    pub fn new(raw: impl Into<String>, target: Arc<dyn OutputTarget>) -> Self {
        Self {
            raw: raw.into(),
            target,
            explicit_format: None,
            format_candidates: Vec::new(),
            file_exists_policy: FileExistsPolicy::default(),
        }
    }

    /// Set the explicit format.
    pub fn with_format(mut self, format: FormatKind) -> Self {
        self.explicit_format = Some(format);
        self
    }

    /// Set the format candidates.
    pub fn with_candidates(mut self, candidates: Vec<FormatKind>) -> Self {
        self.format_candidates = candidates;
        self
    }

    /// Set the file exists policy.
    pub fn with_file_exists_policy(mut self, policy: FileExistsPolicy) -> Self {
        self.file_exists_policy = policy;
        self
    }
}
