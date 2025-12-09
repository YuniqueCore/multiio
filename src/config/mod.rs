//! Configuration types for I/O specifications.
//!
//! This module provides:
//! - `InputSpec`: Specification for a single input source
//! - `OutputSpec`: Specification for a single output target
//! - `FileExistsPolicy`: Policy for handling existing output files
//! - `PipelineConfig`: Configuration for complete I/O pipelines

mod pipeline;
mod spec;

pub use pipeline::{InputConfig, OutputConfig, PipelineConfig};
pub use spec::{FileExistsPolicy, InputSpec, OutputSpec};

// Async versions
#[cfg(feature = "async")]
mod async_spec;

#[cfg(feature = "async")]
pub use async_spec::{AsyncInputSpec, AsyncOutputSpec};
