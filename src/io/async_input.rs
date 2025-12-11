use std::fmt::Debug;

use async_trait::async_trait;
use tokio::io::AsyncRead;

#[async_trait]
pub trait AsyncInputProvider: Send + Sync + Debug {
    /// Returns a unique identifier for this input source.
    fn id(&self) -> &str;

    /// Open and return a new async readable stream.
    async fn open(&self) -> std::io::Result<Box<dyn AsyncRead + Unpin + Send>>;
}
