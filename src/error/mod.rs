//! Error types and policies for multiio I/O operations.
//!
//! This module provides:
//! - `ErrorPolicy`: Controls whether to fail fast or accumulate errors
//! - `Stage`: Indicates where an error occurred in the I/O pipeline
//! - `SingleIoError`: A single I/O error with context
//! - `AggregateError`: A collection of errors when using `Accumulate` policy

use std::fmt;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorPolicy {
    /// Stop at the first error encountered
    FastFail,
    /// Collect all errors and return them together
    #[default]
    Accumulate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    ResolveInput,
    ResolveOutput,
    /// Error while opening the I/O stream
    Open,
    Parse,
    Serialize,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::ResolveInput => write!(f, "ResolveInput"),
            Stage::ResolveOutput => write!(f, "ResolveOutput"),
            Stage::Open => write!(f, "Open"),
            Stage::Parse => write!(f, "Parse"),
            Stage::Serialize => write!(f, "Serialize"),
        }
    }
}

#[derive(Debug)]
pub struct SingleIoError {
    /// Stage where the error occurred
    pub stage: Stage,
    /// Identifier of the target (file path, "-" for stdin/stdout, etc.)
    pub target: String,
    /// The underlying error
    pub error: Box<dyn std::error::Error + Send + Sync>,
}

impl fmt::Display for SingleIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.stage, self.target, self.error)
    }
}

impl std::error::Error for SingleIoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.error.as_ref())
    }
}

/// An aggregate of multiple I/O errors.
///
/// This is returned when using `ErrorPolicy::Accumulate` and multiple errors occurred.
#[derive(Debug, Error)]
pub struct AggregateError {
    /// Collection of individual errors
    pub errors: Vec<SingleIoError>,
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "I/O encountered {} error(s):", self.errors.len())?;
        for (i, e) in self.errors.iter().enumerate() {
            writeln!(f, "  #{}: {}", i + 1, e)?;
        }
        Ok(())
    }
}

impl AggregateError {
    /// Create a new aggregate error with a single error.
    pub fn single(error: SingleIoError) -> Self {
        Self {
            errors: vec![error],
        }
    }

    /// Check if there are no errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl From<SingleIoError> for AggregateError {
    fn from(error: SingleIoError) -> Self {
        Self::single(error)
    }
}

#[cfg(feature = "miette")]
mod miette_impl;

#[cfg(feature = "miette")]
pub use miette_impl::*;
