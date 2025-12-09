//! CLI integration helpers for multiio.
//!
//! This module provides utilities to integrate multiio with CLI argument parsers
//! like `clap`. It helps convert CLI arguments into multiio configurations.
//!
//! # Example with clap
//!
//! ```rust,ignore
//! use clap::Parser;
//! use multiio::cli::{InputArgs, OutputArgs};
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[clap(flatten)]
//!     input: InputArgs,
//!
//!     #[clap(flatten)]
//!     output: OutputArgs,
//! }
//!
//! fn main() {
//!     let cli = Cli::parse();
//!
//!     let builder = MultiioBuilder::new(default_registry())
//!         .with_input_args(&cli.input)
//!         .with_output_args(&cli.output);
//! }
//! ```

use crate::format::FormatKind;

/// Common input arguments for CLI applications.
///
/// Can be used with `#[clap(flatten)]` to add standard input options.
#[derive(Debug, Clone, Default)]
pub struct InputArgs {
    /// Input file paths. Use "-" for stdin.
    pub inputs: Vec<String>,
    /// Explicit input format (overrides auto-detection).
    pub input_format: Option<String>,
}

impl InputArgs {
    /// Create new empty input arguments.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input path.
    pub fn with_input(mut self, path: impl Into<String>) -> Self {
        self.inputs.push(path.into());
        self
    }

    /// Set explicit input format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.input_format = Some(format.into());
        self
    }

    /// Parse the format string into FormatKind.
    pub fn format_kind(&self) -> Option<FormatKind> {
        self.input_format
            .as_ref()
            .and_then(|s| FormatKind::from_str(s))
    }

    /// Check if reading from stdin.
    pub fn is_stdin(&self) -> bool {
        self.inputs.iter().any(|s| s == "-")
    }
}

/// Common output arguments for CLI applications.
#[derive(Debug, Clone, Default)]
pub struct OutputArgs {
    /// Output file paths. Use "-" for stdout.
    pub outputs: Vec<String>,
    /// Explicit output format (overrides auto-detection).
    pub output_format: Option<String>,
    /// What to do if output file exists.
    pub overwrite: bool,
    /// Append to existing file instead of overwriting.
    pub append: bool,
}

impl OutputArgs {
    /// Create new empty output arguments.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an output path.
    pub fn with_output(mut self, path: impl Into<String>) -> Self {
        self.outputs.push(path.into());
        self
    }

    /// Set explicit output format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = Some(format.into());
        self
    }

    /// Enable overwrite mode.
    pub fn with_overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Enable append mode.
    pub fn with_append(mut self) -> Self {
        self.append = true;
        self
    }

    /// Parse the format string into FormatKind.
    pub fn format_kind(&self) -> Option<FormatKind> {
        self.output_format
            .as_ref()
            .and_then(|s| FormatKind::from_str(s))
    }

    /// Check if writing to stdout.
    pub fn is_stdout(&self) -> bool {
        self.outputs.iter().any(|s| s == "-")
    }

    /// Get the file exists policy based on flags.
    pub fn file_exists_policy(&self) -> crate::config::FileExistsPolicy {
        if self.append {
            crate::config::FileExistsPolicy::Append
        } else if self.overwrite {
            crate::config::FileExistsPolicy::Overwrite
        } else {
            crate::config::FileExistsPolicy::Error
        }
    }
}

/// Parse a format string into a FormatKind.
///
/// Supports common format names and aliases.
pub fn parse_format(s: &str) -> Option<FormatKind> {
    FormatKind::from_str(s)
}

/// Infer format from file extension.
pub fn infer_format_from_path(path: &str) -> Option<FormatKind> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())?;
    FormatKind::from_str(ext)
}
