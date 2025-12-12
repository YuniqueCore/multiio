//! # multiio
//!
//! A unified I/O orchestration library for CLI and server applications.
//!
//! ## Overview
//!
//! multiio provides:
//! - **Multi-input/Multi-output**: Read from and write to multiple sources simultaneously
//! - **Format abstraction**: Built-in support for JSON, YAML, CSV, XML, and plaintext
//! - **Extensible formats**: Implement the `Format` trait for custom formats
//! - **Sync and Async**: Both synchronous and asynchronous I/O support
//! - **Error handling**: Configurable error policies (FastFail or Accumulate)
//! - **Pipeline configuration**: Define I/O workflows via YAML/JSON config files
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use multiio::{MultiioBuilder, error::ErrorPolicy};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct Config {
//!     name: String,
//!     value: i32,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = MultiioBuilder::default()
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
//! - `plaintext` - Plaintext format support (enabled by default)
//! - `async` - Async I/O support with Tokio
//! - `miette` - Pretty error reporting with miette
//!
//! ## Streaming usage & semantics
//!
//! multiio provides streaming deserialization for several formats.
//!
//! - **Sync streaming helpers** (in `multiio::format`):
//!   - `deserialize_json_stream` – NDJSON / multi-document JSON (`Read` -> `Iterator`)
//!   - `deserialize_csv_stream` – row-by-row CSV records
//!   - `deserialize_yaml_stream` – multi-document YAML
//!   - `deserialize_plaintext_stream` – line-based plaintext
//! - **Sync engine streaming**:
//!   - `IoEngine::read_records<T>()` uses `FormatRegistry::stream_deserialize_into` to
//!     pick the right streaming implementation (including custom formats). Each record
//!     is yielded as `Result<T, SingleIoError>`.
//! - **Memory model (sync)**:
//!   - Streaming helpers work directly from a `Read` implementation and do not require
//!     loading the entire input into memory at once, aside from what the underlying
//!     decoder (e.g. `serde_json`, `csv`, `serde_yaml`) buffers internally.
//!
//! - **Async engine streaming**:
//!   - `AsyncIoEngine::read_records_async<T>(concurrency)` reads each async input into
//!     a `Vec<u8>` and then reuses the same sync streaming helpers via an in-memory
//!     cursor. This gives record-level streaming semantics on top of an async source,
//!     while keeping the implementation simple and predictable.
//!   - `concurrency` controls how many inputs are processed in parallel; records from
//!     different inputs may be interleaved in the resulting stream.
//! - **Memory model (async)**:
//!   - Because each input is first read into a `Vec<u8>`, the peak memory usage per
//!     input is still proportional to the full input size. Streaming at the record
//!     level improves processing behavior, but does not yet provide true incremental
//!     I/O at the byte level.
//!
//! - **YAML async streaming note**:
//!   - Synchronous YAML streaming (`deserialize_yaml_stream`) yields documents lazily
//!     from a reader.
//!   - In the async engine, YAML streaming currently collects all documents from a
//!     reader into an in-memory `Vec` before exposing them as a record stream. This
//!     avoids Send-related limitations in the underlying `serde_yaml` streaming
//!     implementation, at the cost of higher temporary memory usage for very large
//!     YAML streams.

// Core modules
pub mod builder;
pub mod cli;
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
#[cfg(feature = "custom")]
pub use format::CustomFormat;
pub use format::{FormatError, FormatKind, FormatRegistry, default_registry};
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
pub use format::{AsyncFormatRegistry, default_async_registry};
#[cfg(feature = "async")]
pub use io::{
    AsyncFileInput, AsyncFileOutput, AsyncInputProvider, AsyncOutputTarget, AsyncStdinInput,
    AsyncStdoutOutput,
};

/// Build a synchronous IoEngine from a PipelineConfig using the default
/// FormatRegistry.
pub fn build_engine_from_pipeline(config: PipelineConfig) -> Result<IoEngine, AggregateError> {
    let registry = format::default_registry();
    builder::MultiioBuilder::from_pipeline_config(config, registry)?.build()
}

/// Build a synchronous IoEngine from a PipelineConfig, allowing the caller to
/// further customize the MultiioBuilder before it is built. This is a natural
/// hook point for registering custom formats or tweaking options based on the
/// parsed configuration.
pub fn build_engine_from_pipeline_with<F>(
    config: PipelineConfig,
    customize: F,
) -> Result<IoEngine, AggregateError>
where
    F: FnOnce(builder::MultiioBuilder) -> builder::MultiioBuilder,
{
    let registry = format::default_registry();
    let builder = builder::MultiioBuilder::from_pipeline_config(config, registry)?;
    let builder = customize(builder);
    builder.build()
}

// Miette re-exports
#[cfg(feature = "miette")]
pub use error::IoDiagnostic;

// Internal test modules (see src/tests)
#[cfg(test)]
mod tests;
