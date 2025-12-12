use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{FormatError, FormatKind};

/// Async deserialize from bytes using the specified format.
pub async fn deserialize_async<T: DeserializeOwned + Send>(
    kind: FormatKind,
    bytes: &[u8],
) -> Result<T, FormatError> {
    super::deserialize(kind, bytes)
}

/// Async serialize to bytes using the specified format.
pub async fn serialize_async<T: Serialize + Sync>(
    kind: FormatKind,
    value: &T,
) -> Result<Vec<u8>, FormatError> {
    super::serialize(kind, value)
}

/// Async deserialize from an async reader.
pub async fn deserialize_from_async_reader<T: DeserializeOwned + Send>(
    kind: FormatKind,
    reader: &mut (dyn AsyncRead + Unpin + Send),
) -> Result<T, FormatError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    deserialize_async(kind, &bytes).await
}

/// Async serialize to an async writer.
pub async fn serialize_to_async_writer<T: Serialize + Sync>(
    kind: FormatKind,
    value: &T,
    writer: &mut (dyn AsyncWrite + Unpin + Send),
) -> Result<(), FormatError> {
    let bytes = serialize_async(kind, value).await?;
    writer.write_all(&bytes).await?;
    Ok(())
}

/// Async format registry (mirrors sync FormatRegistry).
#[derive(Debug, Clone)]
pub struct AsyncFormatRegistry {
    formats: Vec<FormatKind>,
}

impl Default for AsyncFormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncFormatRegistry {
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    /// Register a format.
    pub fn register(&mut self, kind: FormatKind) {
        if !self.formats.contains(&kind) {
            self.formats.push(kind);
        }
    }

    /// Register a format (builder pattern).
    pub fn with_format(mut self, kind: FormatKind) -> Self {
        self.register(kind);
        self
    }

    /// Check if a format is registered.
    pub fn has_format(&self, kind: &FormatKind) -> bool {
        self.formats.contains(kind)
    }

    /// Get format kind for a file extension.
    pub fn kind_for_extension(&self, ext: &str) -> Option<FormatKind> {
        let ext_lower = ext.to_ascii_lowercase();
        for kind in &self.formats {
            if kind
                .extensions()
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&ext_lower))
            {
                return Some(*kind);
            }
        }
        None
    }

    /// Resolve a format based on explicit kind or candidates.
    pub fn resolve(
        &self,
        explicit: Option<&FormatKind>,
        candidates: &[FormatKind],
    ) -> Result<FormatKind, FormatError> {
        if let Some(k) = explicit {
            if self.has_format(k) && k.is_available() {
                return Ok(*k);
            }
            return Err(FormatError::UnknownFormat(*k));
        }
        for k in candidates {
            if self.has_format(k) && k.is_available() {
                return Ok(*k);
            }
        }
        Err(FormatError::NoFormatMatched)
    }

    /// Get all registered formats.
    pub fn formats(&self) -> &[FormatKind] {
        &self.formats
    }
}

/// Create a default async registry with all enabled formats.
pub fn default_async_registry() -> AsyncFormatRegistry {
    let mut registry = AsyncFormatRegistry::new();
    super::for_each_enabled_builtin(|k| registry.register(k));
    registry
}
