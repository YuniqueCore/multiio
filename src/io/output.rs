//! Output target trait definition.

use std::fmt::Debug;
use std::io::Write;

/// Trait for synchronous output targets.
///
/// Implementors provide a way to open a writable stream to various destinations
/// such as files, stdout/stderr, or in-memory buffers.
pub trait OutputTarget: Send + Sync + Debug {
    /// Returns a unique identifier for this output target.
    ///
    /// This is used for error messages and logging.
    /// Convention: "-" for stdout, file path for files.
    fn id(&self) -> &str;

    /// Open the target for writing, truncating any existing content.
    fn open_overwrite(&self) -> std::io::Result<Box<dyn Write + Send>>;

    /// Open the target for appending to existing content.
    fn open_append(&self) -> std::io::Result<Box<dyn Write + Send>>;
}
