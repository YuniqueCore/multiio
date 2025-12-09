//! Input provider trait definition.

use std::fmt::Debug;
use std::io::Read;

/// Trait for synchronous input providers.
///
/// Implementors provide a way to open a readable stream from various sources
/// such as files, stdin, network, or in-memory buffers.
pub trait InputProvider: Send + Sync + Debug {
    /// Returns a unique identifier for this input source.
    ///
    /// This is used for error messages and logging.
    /// Convention: "-" for stdin, file path for files.
    fn id(&self) -> &str;

    /// Open and return a new readable stream.
    ///
    /// Each call should return a fresh stream positioned at the beginning.
    fn open(&self) -> std::io::Result<Box<dyn Read + Send>>;
}
