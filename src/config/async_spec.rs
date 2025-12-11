use std::sync::Arc;

use crate::format::FormatKind;
use crate::io::{AsyncInputProvider, AsyncOutputTarget};

use super::FileExistsPolicy;

#[derive(Debug, Clone)]
pub struct AsyncInputSpec {
    /// Raw input argument or configuration string
    pub raw: String,
    /// The async input provider implementation
    pub provider: Arc<dyn AsyncInputProvider>,
    /// Explicitly specified format (if any)
    pub explicit_format: Option<FormatKind>,
    /// Candidate formats to try (in order)
    pub format_candidates: Vec<FormatKind>,
}

impl AsyncInputSpec {
    pub fn new(raw: impl Into<String>, provider: Arc<dyn AsyncInputProvider>) -> Self {
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

#[derive(Debug, Clone)]
pub struct AsyncOutputSpec {
    /// Raw output argument or configuration string
    pub raw: String,
    /// The async output target implementation
    pub target: Arc<dyn AsyncOutputTarget>,
    /// Explicitly specified format (if any)
    pub explicit_format: Option<FormatKind>,
    /// Candidate formats to try (in order)
    pub format_candidates: Vec<FormatKind>,
    /// Policy for handling existing files
    pub file_exists_policy: FileExistsPolicy,
}

impl AsyncOutputSpec {
    pub fn new(raw: impl Into<String>, target: Arc<dyn AsyncOutputTarget>) -> Self {
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
