//! Format abstraction for serialization and deserialization.
//!
//! This module provides:
//! - `FormatKind`: Enum representing different data formats
//! - `FormatError`: Errors that can occur during format operations
//! - `FormatRegistry`: Registry managing formats by kind
//! - `CustomFormat`: Support for user-defined custom formats

use std::io::Read;

mod custom;
pub use custom::CustomFormat;

// Per-format implementations
#[cfg(feature = "csv")]
mod csv;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "markdown")]
mod markdown;
#[cfg(feature = "plaintext")]
mod plaintext;
#[cfg(feature = "xml")]
mod xml;
#[cfg(feature = "yaml")]
mod yaml;

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

/// Represents different data format types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FormatKind {
    /// Plain text format
    Plaintext,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// XML format
    Xml,
    /// CSV format
    Csv,
    /// Markdown format
    Markdown,
    /// Custom format with a unique name
    Custom(&'static str),
}

impl std::fmt::Display for FormatKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatKind::Plaintext => write!(f, "plaintext"),
            FormatKind::Json => write!(f, "json"),
            FormatKind::Yaml => write!(f, "yaml"),
            FormatKind::Xml => write!(f, "xml"),
            FormatKind::Csv => write!(f, "csv"),
            FormatKind::Markdown => write!(f, "markdown"),
            FormatKind::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl Copy for FormatKind {}

// Manual Copy implementation doesn't work with &'static str, use Clone instead
impl FormatKind {
    /// Create a custom format kind with the given name.
    pub fn custom(name: &'static str) -> Self {
        FormatKind::Custom(name)
    }
}

impl std::str::FromStr for FormatKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();

        if let Some(rest) = lower.strip_prefix("custom:") {
            // Leak the custom format name into a 'static str so it can live
            // inside FormatKind::Custom. This is acceptable for configuration-
            // level strings which are created once per process.
            let leaked: &'static str = Box::leak(rest.to_string().into_boxed_str());
            return Ok(FormatKind::Custom(leaked));
        }

        let kind = match lower.as_str() {
            "plaintext" | "text" | "txt" => FormatKind::Plaintext,
            "json" => FormatKind::Json,
            "yaml" | "yml" => FormatKind::Yaml,
            "xml" => FormatKind::Xml,
            "csv" => FormatKind::Csv,
            "markdown" | "md" => FormatKind::Markdown,
            _ => return Err(()),
        };
        Ok(kind)
    }
}

impl FormatKind {
    /// Get file extensions for this format.
    /// Note: For custom formats, this returns an empty slice.
    /// Use FormatRegistry to get extensions for custom formats.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            FormatKind::Plaintext => &["txt", "text"],
            FormatKind::Json => &["json"],
            FormatKind::Yaml => &["yaml", "yml"],
            FormatKind::Xml => &["xml"],
            FormatKind::Csv => &["csv"],
            FormatKind::Markdown => &["md", "markdown"],
            FormatKind::Custom(_) => &[],
        }
    }

    /// Check if this format is available (feature enabled).
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "json")]
            FormatKind::Json => true,
            #[cfg(not(feature = "json"))]
            FormatKind::Json => false,

            #[cfg(feature = "yaml")]
            FormatKind::Yaml => true,
            #[cfg(not(feature = "yaml"))]
            FormatKind::Yaml => false,

            #[cfg(feature = "csv")]
            FormatKind::Csv => true,
            #[cfg(not(feature = "csv"))]
            FormatKind::Csv => false,

            #[cfg(feature = "xml")]
            FormatKind::Xml => true,
            #[cfg(not(feature = "xml"))]
            FormatKind::Xml => false,

            #[cfg(feature = "markdown")]
            FormatKind::Markdown => true,
            #[cfg(not(feature = "markdown"))]
            FormatKind::Markdown => false,

            #[cfg(feature = "plaintext")]
            FormatKind::Plaintext => true,
            #[cfg(not(feature = "plaintext"))]
            FormatKind::Plaintext => false,

            // Custom formats are always considered available
            // (availability is determined by registration)
            FormatKind::Custom(_) => true,
        }
    }
}

/// Errors that can occur during format operations.
#[derive(Debug, Error)]
pub enum FormatError {
    /// The requested format is unknown or not registered
    #[error("Unknown format: {0}")]
    UnknownFormat(FormatKind),

    /// No format matched the input
    #[error("No format matched the input")]
    NoFormatMatched,

    /// Format feature not enabled
    #[error("Format '{0}' is not enabled. Enable the corresponding feature.")]
    NotEnabled(FormatKind),

    /// I/O error during format operation
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serde error: {0}")]
    Serde(Box<dyn std::error::Error + Send + Sync>),

    /// Other format-specific error
    #[error("Format error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Deserialize from bytes using the specified format.
pub fn deserialize<T: DeserializeOwned>(kind: FormatKind, bytes: &[u8]) -> Result<T, FormatError> {
    match kind {
        #[cfg(feature = "json")]
        FormatKind::Json => json::deserialize(bytes),

        #[cfg(feature = "yaml")]
        FormatKind::Yaml => yaml::deserialize(bytes),

        #[cfg(feature = "csv")]
        FormatKind::Csv => csv::deserialize(bytes),

        #[cfg(feature = "xml")]
        FormatKind::Xml => xml::deserialize(bytes),

        #[cfg(feature = "markdown")]
        FormatKind::Markdown => markdown::deserialize(bytes),

        #[cfg(feature = "plaintext")]
        FormatKind::Plaintext => plaintext::deserialize(bytes),

        #[allow(unreachable_patterns)]
        _ => Err(FormatError::NotEnabled(kind)),
    }
}

/// Serialize to bytes using the specified format.
pub fn serialize<T: Serialize>(kind: FormatKind, value: &T) -> Result<Vec<u8>, FormatError> {
    match kind {
        #[cfg(feature = "json")]
        FormatKind::Json => json::serialize(value),

        #[cfg(feature = "yaml")]
        FormatKind::Yaml => yaml::serialize(value),

        #[cfg(feature = "csv")]
        FormatKind::Csv => csv::serialize(value),

        #[cfg(feature = "xml")]
        FormatKind::Xml => xml::serialize(value),

        #[cfg(feature = "markdown")]
        FormatKind::Markdown => markdown::serialize(value),

        #[cfg(feature = "plaintext")]
        FormatKind::Plaintext => plaintext::serialize(value),

        #[allow(unreachable_patterns)]
        _ => Err(FormatError::NotEnabled(kind)),
    }
}

/// Deserialize from a reader using the specified format.
pub fn deserialize_from_reader<T: DeserializeOwned>(
    kind: FormatKind,
    reader: &mut dyn Read,
) -> Result<T, FormatError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    deserialize(kind, &bytes)
}

/// Stream JSON values from a reader as multiple top-level JSON documents.
#[cfg(feature = "json")]
pub fn deserialize_json_stream<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read,
{
    json::stream_deserialize(reader)
}

/// Stream CSV records from a reader.
#[cfg(feature = "csv")]
pub fn deserialize_csv_stream<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read,
{
    csv::stream_deserialize(reader)
}

/// Stream YAML documents from a reader.
#[cfg(feature = "yaml")]
pub fn deserialize_yaml_stream<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read + 'static,
{
    yaml::stream_deserialize(reader)
}

/// Stream plaintext records (typically lines) from a reader.
#[cfg(feature = "plaintext")]
pub fn deserialize_plaintext_stream<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read,
{
    plaintext::stream_deserialize(reader)
}

/// Format registry.
#[derive(Default)]
pub struct FormatRegistry {
    /// Registered built-in formats.
    formats: Vec<FormatKind>,
    /// Custom format handlers
    custom_formats: Vec<CustomFormat>,
}

impl FormatRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
            custom_formats: Vec::new(),
        }
    }

    /// Register a built-in format.
    pub fn register(&mut self, kind: FormatKind) {
        if !self.formats.contains(&kind) {
            self.formats.push(kind);
        }
    }

    /// Register a built-in format (builder pattern).
    pub fn with_format(mut self, kind: FormatKind) -> Self {
        self.register(kind);
        self
    }

    /// Register a custom format handler.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use multiio::format::{CustomFormat, FormatRegistry, FormatError};
    ///
    /// let mut registry = FormatRegistry::new();
    /// registry.register_custom(
    ///     CustomFormat::new("toml", &["toml"])
    ///         .with_deserialize(|bytes| {
    ///             // Your deserialization logic
    ///             Ok(serde_json::Value::Null)
    ///         })
    ///         .with_serialize(|value| {
    ///             // Your serialization logic
    ///             Ok(Vec::new())
    ///         })
    /// );
    /// ```
    pub fn register_custom(&mut self, format: CustomFormat) {
        // Also register the FormatKind::Custom variant
        let kind = FormatKind::Custom(format.name);
        if !self.formats.contains(&kind) {
            self.formats.push(kind);
        }
        self.custom_formats.push(format);
    }

    /// Register a custom format handler (builder pattern).
    pub fn with_custom_format(mut self, format: CustomFormat) -> Self {
        self.register_custom(format);
        self
    }

    /// Check if a format is registered.
    pub fn has_format(&self, kind: &FormatKind) -> bool {
        self.formats.contains(kind)
    }

    /// Get the custom format handler for a format kind.
    pub fn get_custom(&self, name: &str) -> Option<&CustomFormat> {
        self.custom_formats.iter().find(|f| f.name == name)
    }

    /// Get format kind for a file extension.
    pub fn kind_for_extension(&self, ext: &str) -> Option<FormatKind> {
        let ext_lower = ext.to_ascii_lowercase();

        // Check built-in formats first
        for kind in &self.formats {
            if kind
                .extensions()
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&ext_lower))
            {
                return Some(*kind);
            }
        }

        // Check custom formats
        for custom in &self.custom_formats {
            if custom.matches_extension(&ext_lower) {
                return Some(FormatKind::Custom(custom.name));
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

    /// Get all registered format kinds.
    pub fn formats(&self) -> &[FormatKind] {
        &self.formats
    }

    /// Get all registered custom formats.
    pub fn custom_formats(&self) -> &[CustomFormat] {
        &self.custom_formats
    }

    /// Deserialize using this registry.
    ///
    /// Automatically handles both built-in and custom formats.
    pub fn deserialize_value<T: DeserializeOwned>(
        &self,
        explicit: Option<&FormatKind>,
        candidates: &[FormatKind],
        bytes: &[u8],
    ) -> Result<T, FormatError> {
        let kind = self.resolve(explicit, candidates)?;

        // Handle custom formats
        if let FormatKind::Custom(name) = &kind {
            let custom = self
                .get_custom(name)
                .ok_or_else(|| FormatError::UnknownFormat(kind))?;
            return custom.deserialize(bytes);
        }

        // Handle built-in formats
        deserialize(kind, bytes)
    }

    /// Serialize using this registry.
    ///
    /// Automatically handles both built-in and custom formats.
    pub fn serialize_value<T: Serialize>(
        &self,
        explicit: Option<&FormatKind>,
        candidates: &[FormatKind],
        value: &T,
    ) -> Result<Vec<u8>, FormatError> {
        let kind = self.resolve(explicit, candidates)?;

        // Handle custom formats
        if let FormatKind::Custom(name) = &kind {
            let custom = self
                .get_custom(name)
                .ok_or_else(|| FormatError::UnknownFormat(kind))?;
            return custom.serialize(value);
        }

        // Handle built-in formats
        serialize(kind, value)
    }

    /// Stream-deserialize values into `T` using this registry.
    ///
    /// For built-in JSON/CSV formats, this uses native streaming decoders.
    /// For custom formats, if a streaming handler is provided it will be used.
    /// Otherwise, falls back to non-streaming deserialization as a single item.
    pub fn stream_deserialize_into<T>(
        &self,
        explicit: Option<&FormatKind>,
        candidates: &[FormatKind],
        reader: Box<dyn Read>,
    ) -> Result<Box<dyn Iterator<Item = Result<T, FormatError>>>, FormatError>
    where
        T: DeserializeOwned + 'static,
    {
        let kind = self.resolve(explicit, candidates)?;

        if let FormatKind::Json = kind {
            #[cfg(feature = "json")]
            {
                let iter = crate::format::deserialize_json_stream::<T, _>(reader);
                return Ok(Box::new(iter));
            }
            #[cfg(not(feature = "json"))]
            {
                return Err(FormatError::NotEnabled(kind));
            }
        }

        if let FormatKind::Csv = kind {
            #[cfg(feature = "csv")]
            {
                let iter = crate::format::deserialize_csv_stream::<T, _>(reader);
                return Ok(Box::new(iter));
            }
            #[cfg(not(feature = "csv"))]
            {
                return Err(FormatError::NotEnabled(kind));
            }
        }

        if let FormatKind::Yaml = kind {
            #[cfg(feature = "yaml")]
            {
                let iter = crate::format::deserialize_yaml_stream::<T, _>(reader);
                return Ok(Box::new(iter));
            }
            #[cfg(not(feature = "yaml"))]
            {
                return Err(FormatError::NotEnabled(kind));
            }
        }

        if let FormatKind::Plaintext = kind {
            #[cfg(feature = "plaintext")]
            {
                let iter = crate::format::deserialize_plaintext_stream::<T, _>(reader);
                return Ok(Box::new(iter));
            }
            #[cfg(not(feature = "plaintext"))]
            {
                return Err(FormatError::NotEnabled(kind));
            }
        }

        if let FormatKind::Custom(name) = kind {
            let custom = self
                .get_custom(name)
                .ok_or_else(|| FormatError::UnknownFormat(kind))?;

            if custom.stream_deserialize_fn.is_some() {
                let iter = custom.stream_deserialize_values(reader)?.map(|res| {
                    res.and_then(|value| {
                        serde_json::from_value::<T>(value)
                            .map_err(|e| FormatError::Serde(Box::new(e)))
                    })
                });
                return Ok(Box::new(iter));
            } else {
                // Fallback: non-streaming, single item
                let mut r = reader;
                let mut bytes = Vec::new();
                r.read_to_end(&mut bytes)?;
                let value = custom.deserialize::<T>(&bytes)?;
                return Ok(Box::new(std::iter::once(Ok(value))));
            }
        }

        // Other built-in formats: fallback to non-streaming, single item
        let mut r = reader;
        let mut bytes = Vec::new();
        r.read_to_end(&mut bytes)?;
        let value = deserialize::<T>(kind, &bytes)?;
        Ok(Box::new(std::iter::once(Ok(value))))
    }
}

/// Create a default registry with all enabled formats.
pub fn default_registry() -> FormatRegistry {
    let mut registry = FormatRegistry::new();

    #[cfg(feature = "json")]
    registry.register(FormatKind::Json);

    #[cfg(feature = "yaml")]
    registry.register(FormatKind::Yaml);

    #[cfg(feature = "plaintext")]
    registry.register(FormatKind::Plaintext);

    #[cfg(feature = "csv")]
    registry.register(FormatKind::Csv);

    #[cfg(feature = "xml")]
    registry.register(FormatKind::Xml);

    #[cfg(feature = "markdown")]
    registry.register(FormatKind::Markdown);

    registry
}

// Async format support
#[cfg(feature = "async")]
mod async_format;

#[cfg(feature = "async")]
pub use async_format::*;
