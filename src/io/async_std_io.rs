use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter};

use super::{AsyncInputProvider, AsyncOutputTarget};

#[derive(Debug, Clone)]
pub struct AsyncStdinInput {
    id: String,
}

impl AsyncStdinInput {
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
        Ok(Box::new(BufReader::new(tokio::io::stdin())))
    }
}

#[derive(Debug, Clone)]
pub struct AsyncFileInput {
    id: String,
    path: PathBuf,
}

impl AsyncFileInput {
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
        Ok(Box::new(BufReader::new(file)))
    }
}

#[derive(Debug, Clone)]
pub struct AsyncStdoutOutput {
    id: String,
}

impl AsyncStdoutOutput {
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
        Ok(Box::new(BufWriter::new(tokio::io::stdout())))
    }

    async fn open_append(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        Ok(Box::new(BufWriter::new(tokio::io::stdout())))
    }
}

#[derive(Debug, Clone)]
pub struct AsyncStderrOutput {
    id: String,
}

impl AsyncStderrOutput {
    pub fn new() -> Self {
        Self {
            id: "stderr".into(),
        }
    }
}

impl Default for AsyncStderrOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AsyncOutputTarget for AsyncStderrOutput {
    fn id(&self) -> &str {
        &self.id
    }

    async fn open_overwrite(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        Ok(Box::new(BufWriter::new(tokio::io::stderr())))
    }

    async fn open_append(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        Ok(Box::new(BufWriter::new(tokio::io::stderr())))
    }
}

#[derive(Debug, Clone)]
pub struct AsyncFileOutput {
    id: String,
    path: PathBuf,
}

impl AsyncFileOutput {
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
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).await?;
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .await?;
        Ok(Box::new(file))
    }

    async fn open_append(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).await?;
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        Ok(Box::new(file))
    }
}
