//! Miette integration for pretty error reporting.

use miette::{Diagnostic, Severity};
use thiserror::Error;

use super::{AggregateError, SingleIoError};

/// A diagnostic wrapper for I/O errors compatible with miette.
#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
pub struct IoDiagnostic {
    /// The error message
    pub message: String,

    #[source]
    /// The underlying error source
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,

    #[help]
    /// Help text for the user
    pub help: Option<String>,

    #[diagnostic(severity)]
    /// Severity level
    pub severity: Severity,
}

impl From<SingleIoError> for IoDiagnostic {
    fn from(e: SingleIoError) -> Self {
        IoDiagnostic {
            message: format!("[{}] on '{}'", e.stage, e.target),
            source: Some(e.error),
            help: Some("Check your I/O arguments and formats".into()),
            severity: Severity::Error,
        }
    }
}

impl From<AggregateError> for miette::Report {
    fn from(agg: AggregateError) -> Self {
        let first = agg.errors.into_iter().next();
        let diag = if let Some(e) = first {
            IoDiagnostic::from(e)
        } else {
            IoDiagnostic {
                message: "Unknown I/O error".into(),
                source: None,
                help: None,
                severity: Severity::Error,
            }
        };
        miette::Report::new(diag)
    }
}

impl From<AggregateError> for IoDiagnostic {
    fn from(agg: AggregateError) -> Self {
        let first = agg.errors.into_iter().next();
        if let Some(e) = first {
            IoDiagnostic::from(e)
        } else {
            IoDiagnostic {
                message: "Unknown I/O error".into(),
                source: None,
                help: None,
                severity: Severity::Error,
            }
        }
    }
}
