//! Custom format support for user-defined formats.
//!
//! This module allows developers to register their own format implementations
//! without modifying the core library.

use std::sync::Arc;

use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

/// Type alias for custom deserialize function.
///
/// Takes raw bytes and returns a `serde_json::Value` which can then be
/// converted to the target type.
pub type DeserializeFn = Arc<dyn Fn(&[u8]) -> Result<serde_json::Value, FormatError> + Send + Sync>;

/// Type alias for custom serialize function.
///
/// Takes a `serde_json::Value` and returns serialized bytes.
pub type SerializeFn =
    Arc<dyn Fn(&serde_json::Value) -> Result<Vec<u8>, FormatError> + Send + Sync>;

/// A custom format handler that can be registered with the FormatRegistry.
///
/// # Example
///
/// ```rust,ignore
/// use multiio::format::{CustomFormat, FormatError};
///
/// let toml_format = CustomFormat::new("toml", &["toml"])
///     .with_deserialize(|bytes| {
///         let s = String::from_utf8_lossy(bytes);
///         toml::from_str(&s)
///             .map_err(|e| FormatError::Serde(Box::new(e)))
///     })
///     .with_serialize(|value| {
///         toml::to_string_pretty(value)
///             .map(|s| s.into_bytes())
///             .map_err(|e| FormatError::Serde(Box::new(e)))
///     });
///
/// registry.register_custom(toml_format);
/// ```
#[derive(Clone)]
pub struct CustomFormat {
    /// Unique name for this format
    pub name: &'static str,
    /// File extensions associated with this format
    pub extensions: &'static [&'static str],
    /// Deserialize function
    pub deserialize_fn: Option<DeserializeFn>,
    /// Serialize function
    pub serialize_fn: Option<SerializeFn>,
}

impl std::fmt::Debug for CustomFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomFormat")
            .field("name", &self.name)
            .field("extensions", &self.extensions)
            .field("has_deserialize", &self.deserialize_fn.is_some())
            .field("has_serialize", &self.serialize_fn.is_some())
            .finish()
    }
}

impl CustomFormat {
    /// Create a new custom format with the given name and extensions.
    pub fn new(name: &'static str, extensions: &'static [&'static str]) -> Self {
        Self {
            name,
            extensions,
            deserialize_fn: None,
            serialize_fn: None,
        }
    }

    /// Set the deserialize function.
    pub fn with_deserialize<F>(mut self, f: F) -> Self
    where
        F: Fn(&[u8]) -> Result<serde_json::Value, FormatError> + Send + Sync + 'static,
    {
        self.deserialize_fn = Some(Arc::new(f));
        self
    }

    /// Set the serialize function.
    pub fn with_serialize<F>(mut self, f: F) -> Self
    where
        F: Fn(&serde_json::Value) -> Result<Vec<u8>, FormatError> + Send + Sync + 'static,
    {
        self.serialize_fn = Some(Arc::new(f));
        self
    }

    /// Deserialize bytes to a typed value.
    pub fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, FormatError> {
        let deserialize_fn = self.deserialize_fn.as_ref().ok_or_else(|| {
            FormatError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "Custom format '{}' does not support deserialization",
                    self.name
                ),
            )))
        })?;

        let value = deserialize_fn(bytes)?;
        serde_json::from_value(value).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    /// Serialize a typed value to bytes.
    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, FormatError> {
        let serialize_fn = self.serialize_fn.as_ref().ok_or_else(|| {
            FormatError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "Custom format '{}' does not support serialization",
                    self.name
                ),
            )))
        })?;

        let json_value =
            serde_json::to_value(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
        serialize_fn(&json_value)
    }

    /// Check if this format matches the given extension.
    pub fn matches_extension(&self, ext: &str) -> bool {
        let ext_lower = ext.to_ascii_lowercase();
        self.extensions
            .iter()
            .any(|e| e.eq_ignore_ascii_case(&ext_lower))
    }
}
