//! CLI integration helpers for multiio.
//!
//! This module keeps CLI-related types intentionally lightweight so callers can
//! integrate with any argument parser (clap/sarge/argh/…).
//!
//! # Token conventions
//!
//! multiio intentionally supports a small set of conventional "special" tokens
//! for CLI ergonomics:
//!
//! - Inputs:
//!   - `-` or `stdin` => stdin
//!   - `=<content>` => inline content (in-memory input)
//!   - `@<path>` => force treating the value as a file path (useful for
//!     disambiguating reserved tokens)
//! - Outputs:
//!   - `-` or `stdout` => stdout
//!   - `stderr` => stderr
//!   - `@<path>` => force treating the value as a file path (e.g. `@stderr`)
//!
//! # Example
//!
//! ```rust,ignore
//! use multiio::{default_registry, MultiioBuilder};
//! use multiio::cli::{InputArgs, OutputArgs};
//!
//! fn run(inputs: Vec<String>, outputs: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
//!     let input = InputArgs::from(inputs);
//!     let output = OutputArgs::from(outputs);
//!
//!     let engine = MultiioBuilder::new(default_registry())
//!         .with_input_args(&input)
//!         .with_output_args(&output)
//!         .build()?;
//!
//!     let values: Vec<serde_json::Value> = engine.read_all()?;
//!     engine.write_all(&values)?;
//!     Ok(())
//! }
//! ```

#[cfg(feature = "sarge")]
mod sarge;

macro_rules! impls_for {
    (
        $name:ident => $type:path
    ) => {
        impl Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<$type> for $name {
            fn from(inputs: $type) -> Self {
                Self(inputs)
            }
        }

        impl From<$name> for $type {
            fn from(args: $name) -> Self {
                args.0
            }
        }
    };
}

/// Common input arguments for CLI applications.
#[derive(Debug, Clone, Default)]
pub struct InputArgs(Vec<String>);

impl InputArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_input(mut self, path: impl Into<String>) -> Self {
        self.push(path.into());
        self
    }

    pub fn is_stdin(&self) -> bool {
        self.iter()
            .any(|s| s == "-" || s.eq_ignore_ascii_case("stdin"))
    }
}

#[derive(Debug, Clone, Default)]
pub struct OutputArgs(Vec<String>);

impl OutputArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_output(mut self, path: impl Into<String>) -> Self {
        self.push(path.into());
        self
    }

    /// Check if writing to stdout.
    pub fn is_stdout(&self) -> bool {
        self.iter()
            .any(|s| s == "-" || s.eq_ignore_ascii_case("stdout"))
    }

    /// Check if writing to stderr.
    pub fn is_stderr(&self) -> bool {
        self.iter().any(|s| s.eq_ignore_ascii_case("stderr"))
    }
}

impls_for!(InputArgs => Vec<String>);
impls_for!(OutputArgs => Vec<String>);

use std::ops::{Deref, DerefMut};

use crate::format::FormatKind;

/// Parse a format string into a FormatKind.
///
/// Supports common format names and aliases.
pub fn parse_format(s: &str) -> Option<FormatKind> {
    s.parse::<FormatKind>().ok()
}

/// Infer format from file extension.
pub fn infer_format_from_path(path: &str) -> Option<FormatKind> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())?;
    ext.parse::<FormatKind>().ok()
}
