//! I/O abstractions for input providers and output targets.
//!
//! This module provides:
//! - `InputProvider`: Trait for input sources
//! - `OutputTarget`: Trait for output destinations
//! - Standard implementations for files, stdin/stdout
//! - In-memory implementations for testing

mod input;
mod memory;
mod output;
mod std_io;

pub use input::InputProvider;
#[cfg(feature = "async")]
pub use memory::AsyncInMemorySource;
pub use memory::{InMemorySink, InMemorySource};
pub use output::OutputTarget;
pub use std_io::{FileInput, FileOutput, StderrOutput, StdinInput, StdoutOutput};

// Async I/O support
#[cfg(feature = "async")]
mod async_input;
#[cfg(feature = "async")]
mod async_output;
#[cfg(feature = "async")]
mod async_std_io;

#[cfg(feature = "async")]
pub use async_input::AsyncInputProvider;
#[cfg(feature = "async")]
pub use async_output::AsyncOutputTarget;
#[cfg(feature = "async")]
pub use async_std_io::{
    AsyncFileInput, AsyncFileOutput, AsyncStderrOutput, AsyncStdinInput, AsyncStdoutOutput,
};
