//! Standard I/O implementations for files and stdin/stdout.

use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use super::{InputProvider, OutputTarget};

/// Input provider for reading from stdin.
#[derive(Debug, Clone)]
pub struct StdinInput {
    id: String,
}

impl StdinInput {
    /// Create a new stdin input provider.
    pub fn new() -> Self {
        Self { id: "-".into() }
    }
}

impl Default for StdinInput {
    fn default() -> Self {
        Self::new()
    }
}

impl InputProvider for StdinInput {
    fn id(&self) -> &str {
        &self.id
    }

    fn open(&self) -> io::Result<Box<dyn Read + Send>> {
        Ok(Box::new(io::stdin()))
    }
}

/// Input provider for reading from files.
#[derive(Debug, Clone)]
pub struct FileInput {
    id: String,
    path: PathBuf,
}

impl FileInput {
    /// Create a new file input provider.
    pub fn new(path: PathBuf) -> Self {
        let id = path.to_string_lossy().into_owned();
        Self { id, path }
    }

    /// Get the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl InputProvider for FileInput {
    fn id(&self) -> &str {
        &self.id
    }

    fn open(&self) -> io::Result<Box<dyn Read + Send>> {
        let file = std::fs::File::open(&self.path)?;
        Ok(Box::new(file))
    }
}

/// Output target for writing to stdout.
#[derive(Debug, Clone)]
pub struct StdoutOutput {
    id: String,
}

impl StdoutOutput {
    /// Create a new stdout output target.
    pub fn new() -> Self {
        Self { id: "-".into() }
    }
}

impl Default for StdoutOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputTarget for StdoutOutput {
    fn id(&self) -> &str {
        &self.id
    }

    fn open_overwrite(&self) -> io::Result<Box<dyn Write + Send>> {
        Ok(Box::new(io::stdout()))
    }

    fn open_append(&self) -> io::Result<Box<dyn Write + Send>> {
        Ok(Box::new(io::stdout()))
    }
}

/// Output target for writing to stderr.
#[derive(Debug, Clone)]
pub struct StderrOutput {
    id: String,
}

impl StderrOutput {
    /// Create a new stderr output target.
    pub fn new() -> Self {
        Self {
            id: "stderr".into(),
        }
    }
}

impl Default for StderrOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputTarget for StderrOutput {
    fn id(&self) -> &str {
        &self.id
    }

    fn open_overwrite(&self) -> io::Result<Box<dyn Write + Send>> {
        Ok(Box::new(io::stderr()))
    }

    fn open_append(&self) -> io::Result<Box<dyn Write + Send>> {
        Ok(Box::new(io::stderr()))
    }
}

/// Output target for writing to files.
#[derive(Debug, Clone)]
pub struct FileOutput {
    id: String,
    path: PathBuf,
}

impl FileOutput {
    /// Create a new file output target.
    pub fn new(path: PathBuf) -> Self {
        let id = path.to_string_lossy().into_owned();
        Self { id, path }
    }

    /// Get the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl OutputTarget for FileOutput {
    fn id(&self) -> &str {
        &self.id
    }

    fn open_overwrite(&self) -> io::Result<Box<dyn Write + Send>> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)?;
        Ok(Box::new(file))
    }

    fn open_append(&self) -> io::Result<Box<dyn Write + Send>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        Ok(Box::new(file))
    }
}
