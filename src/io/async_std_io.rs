//! Async standard I/O implementations.

use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncRead, AsyncWrite};

use super::{AsyncInputProvider, AsyncOutputTarget};

/// Async input provider for reading from stdin.
#[derive(Debug, Clone)]
pub struct AsyncStdinInput {
    id: String,
}

impl AsyncStdinInput {
    /// Create a new async stdin input provider.
    pub fn new() -> Self {
        Self { id: "-".into() }
    }
}

impl Default for AsyncStdinInput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AsyncInputProvider for AsyncStdinInput {
    fn id(&self) -> &str {
        &self.id
    }

    async fn open(&self) -> std::io::Result<Box<dyn AsyncRead + Unpin + Send>> {
        Ok(Box::new(tokio::io::stdin()))
    }
}

/// Async input provider for reading from files.
#[derive(Debug, Clone)]
pub struct AsyncFileInput {
    id: String,
    path: PathBuf,
}

impl AsyncFileInput {
    /// Create a new async file input provider.
    pub fn new(path: PathBuf) -> Self {
        let id = path.to_string_lossy().into_owned();
        Self { id, path }
    }

    /// Get the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[async_trait]
impl AsyncInputProvider for AsyncFileInput {
    fn id(&self) -> &str {
        &self.id
    }

    async fn open(&self) -> std::io::Result<Box<dyn AsyncRead + Unpin + Send>> {
        let file = tokio::fs::File::open(&self.path).await?;
        Ok(Box::new(file))
    }
}

/// Async output target for writing to stdout.
#[derive(Debug, Clone)]
pub struct AsyncStdoutOutput {
    id: String,
}

impl AsyncStdoutOutput {
    /// Create a new async stdout output target.
    pub fn new() -> Self {
        Self { id: "-".into() }
    }
}

impl Default for AsyncStdoutOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AsyncOutputTarget for AsyncStdoutOutput {
    fn id(&self) -> &str {
        &self.id
    }

    async fn open_overwrite(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        Ok(Box::new(tokio::io::stdout()))
    }

    async fn open_append(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        Ok(Box::new(tokio::io::stdout()))
    }
}

/// Async output target for writing to files.
#[derive(Debug, Clone)]
pub struct AsyncFileOutput {
    id: String,
    path: PathBuf,
}

impl AsyncFileOutput {
    /// Create a new async file output target.
    pub fn new(path: PathBuf) -> Self {
        let id = path.to_string_lossy().into_owned();
        Self { id, path }
    }

    /// Get the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[async_trait]
impl AsyncOutputTarget for AsyncFileOutput {
    fn id(&self) -> &str {
        &self.id
    }

    async fn open_overwrite(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .await?;
        Ok(Box::new(file))
    }

    async fn open_append(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        Ok(Box::new(file))
    }
}
