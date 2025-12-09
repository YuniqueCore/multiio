//! # multiio
//!
//! A unified I/O orchestration library for CLI and server applications.
//!
//! ## Overview
//!
//! multiio provides:
//! - **Multi-input/Multi-output**: Read from and write to multiple sources simultaneously
//! - **Format abstraction**: Built-in support for JSON, YAML, CSV, XML, Markdown, and plaintext
//! - **Extensible formats**: Implement the `Format` trait for custom formats
//! - **Sync and Async**: Both synchronous and asynchronous I/O support
//! - **Error handling**: Configurable error policies (FastFail or Accumulate)
//! - **Pipeline configuration**: Define I/O workflows via YAML/JSON config files
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use multiio::{MultiioBuilder, format::default_registry, error::ErrorPolicy};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct Config {
//!     name: String,
//!     value: i32,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = default_registry();
//!     let engine = MultiioBuilder::new(registry)
//!         .add_input("config.json")
//!         .add_output("-")  // stdout
//!         .with_mode(ErrorPolicy::FastFail)
//!         .build()?;
//!
//!     let configs: Vec<Config> = engine.read_all()?;
//!     engine.write_all(&configs)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - `json` - JSON format support (enabled by default)
//! - `yaml` - YAML format support (enabled by default)
//! - `csv` - CSV format support (enabled by default)
//! - `xml` - XML format support
//! - `markdown` - Markdown format support
//! - `plaintext` - Plaintext format support (enabled by default)
//! - `async` - Async I/O support with Tokio
//! - `miette` - Pretty error reporting with miette

// Core modules
pub mod builder;
pub mod config;
pub mod engine;
pub mod error;
pub mod format;
pub mod io;

// Async modules (feature-gated)
#[cfg(feature = "async")]
pub mod builder_async;
#[cfg(feature = "async")]
pub mod engine_async;

// Re-exports for convenience
pub use builder::MultiioBuilder;
pub use config::{FileExistsPolicy, InputSpec, OutputSpec, PipelineConfig};
pub use engine::IoEngine;
pub use error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
pub use format::{Format, FormatError, FormatKind, FormatRegistry, default_registry};
pub use io::{
    FileInput, FileOutput, InMemorySink, InMemorySource, InputProvider, OutputTarget, StderrOutput,
    StdinInput, StdoutOutput,
};

// Async re-exports
#[cfg(feature = "async")]
pub use builder_async::MultiioAsyncBuilder;
#[cfg(feature = "async")]
pub use config::{AsyncInputSpec, AsyncOutputSpec};
#[cfg(feature = "async")]
pub use engine_async::AsyncIoEngine;
#[cfg(feature = "async")]
pub use format::{AsyncFormat, AsyncFormatRegistry, default_async_registry};
#[cfg(feature = "async")]
pub use io::{
    AsyncFileInput, AsyncFileOutput, AsyncInputProvider, AsyncOutputTarget, AsyncStdinInput,
    AsyncStdoutOutput,
};

// Miette re-exports
#[cfg(feature = "miette")]
pub use error::IoDiagnostic;
