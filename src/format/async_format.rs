//! Async format trait and registry for asynchronous I/O operations.

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{FormatError, FormatKind};

/// Trait for asynchronous format implementations.
#[async_trait]
pub trait AsyncFormat: Send + Sync + 'static {
    /// Returns the kind of this format.
    fn kind(&self) -> FormatKind;

    /// Returns the file extensions associated with this format.
    fn extensions(&self) -> &'static [&'static str];

    /// Deserialize a value from an async reader.
    async fn deserialize<T>(
        &self,
        reader: &mut (dyn AsyncRead + Unpin + Send),
    ) -> Result<T, FormatError>
    where
        T: DeserializeOwned + Send;

    /// Serialize a value to an async writer.
    async fn serialize<T>(
        &self,
        value: &T,
        writer: &mut (dyn AsyncWrite + Unpin + Send),
    ) -> Result<(), FormatError>
    where
        T: Serialize + Sync;
}

/// Registry for async format implementations.
pub struct AsyncFormatRegistry {
    formats: Vec<Box<dyn AsyncFormat>>,
}

impl Default for AsyncFormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncFormatRegistry {
    /// Create a new empty async registry.
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    /// Register an async format implementation.
    pub fn register(&mut self, format: Box<dyn AsyncFormat>) {
        self.formats.push(format);
    }

    /// Register an async format implementation (builder pattern).
    pub fn with_format(mut self, format: Box<dyn AsyncFormat>) -> Self {
        self.register(format);
        self
    }

    /// Get a format by its kind.
    pub fn format_for_kind(&self, kind: &FormatKind) -> Option<&dyn AsyncFormat> {
        self.formats
            .iter()
            .find(|f| &f.kind() == kind)
            .map(|b| b.as_ref())
    }

    /// Get the format kind for a file extension.
    pub fn kind_for_extension(&self, ext: &str) -> Option<FormatKind> {
        let ext_lower = ext.to_ascii_lowercase();
        for f in &self.formats {
            if f.extensions()
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&ext_lower))
            {
                return Some(f.kind());
            }
        }
        None
    }

    /// Resolve a format based on explicit kind or candidates.
    pub fn resolve(
        &self,
        explicit: Option<&FormatKind>,
        candidates: &[FormatKind],
    ) -> Result<&dyn AsyncFormat, FormatError> {
        if let Some(k) = explicit {
            return self
                .format_for_kind(k)
                .ok_or_else(|| FormatError::UnknownFormat(k.clone()));
        }
        for k in candidates {
            if let Some(fmt) = self.format_for_kind(k) {
                return Ok(fmt);
            }
        }
        Err(FormatError::NoFormatMatched)
    }
}

// Async JSON format implementation
#[cfg(feature = "json")]
pub struct AsyncJsonFormat;

#[cfg(feature = "json")]
#[async_trait]
impl AsyncFormat for AsyncJsonFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Json
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["json"]
    }

    async fn deserialize<T>(
        &self,
        reader: &mut (dyn AsyncRead + Unpin + Send),
    ) -> Result<T, FormatError>
    where
        T: DeserializeOwned + Send,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    async fn serialize<T>(
        &self,
        value: &T,
        writer: &mut (dyn AsyncWrite + Unpin + Send),
    ) -> Result<(), FormatError>
    where
        T: Serialize + Sync,
    {
        let bytes =
            serde_json::to_vec_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
        writer.write_all(&bytes).await?;
        Ok(())
    }
}

// Async YAML format implementation
#[cfg(feature = "yaml")]
pub struct AsyncYamlFormat;

#[cfg(feature = "yaml")]
#[async_trait]
impl AsyncFormat for AsyncYamlFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Yaml
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["yaml", "yml"]
    }

    async fn deserialize<T>(
        &self,
        reader: &mut (dyn AsyncRead + Unpin + Send),
    ) -> Result<T, FormatError>
    where
        T: DeserializeOwned + Send,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        serde_yaml::from_slice(&buf).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    async fn serialize<T>(
        &self,
        value: &T,
        writer: &mut (dyn AsyncWrite + Unpin + Send),
    ) -> Result<(), FormatError>
    where
        T: Serialize + Sync,
    {
        let s = serde_yaml::to_string(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
        writer.write_all(s.as_bytes()).await?;
        Ok(())
    }
}

/// Create a default async registry with all enabled formats.
pub fn default_async_registry() -> AsyncFormatRegistry {
    let mut registry = AsyncFormatRegistry::new();

    #[cfg(feature = "json")]
    registry.register(Box::new(AsyncJsonFormat));

    #[cfg(feature = "yaml")]
    registry.register(Box::new(AsyncYamlFormat));

    registry
}
