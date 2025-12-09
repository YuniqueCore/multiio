//! Async output target trait definition.

use std::fmt::Debug;

use async_trait::async_trait;
use tokio::io::AsyncWrite;

/// Trait for asynchronous output targets.
#[async_trait]
pub trait AsyncOutputTarget: Send + Sync + Debug {
    /// Returns a unique identifier for this output target.
    fn id(&self) -> &str;

    /// Open the target for writing, truncating any existing content.
    async fn open_overwrite(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>>;

    /// Open the target for appending to existing content.
    async fn open_append(&self) -> std::io::Result<Box<dyn AsyncWrite + Unpin + Send>>;
}
