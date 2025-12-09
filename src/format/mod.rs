//! Format abstraction for serialization and deserialization.
//!
//! This module provides:
//! - `FormatKind`: Enum representing different data formats
//! - `FormatError`: Errors that can occur during format operations
//! - `FormatRegistry`: Registry managing formats by kind
//! - `CustomFormat`: Support for user-defined custom formats

use std::io::{Read, Write};

mod custom;
pub use custom::CustomFormat;

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

impl FormatKind {
    /// Parse a format kind from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "plaintext" | "text" | "txt" => Some(FormatKind::Plaintext),
            "json" => Some(FormatKind::Json),
            "yaml" | "yml" => Some(FormatKind::Yaml),
            "xml" => Some(FormatKind::Xml),
            "csv" => Some(FormatKind::Csv),
            "markdown" | "md" => Some(FormatKind::Markdown),
            _ => None,
        }
    }

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
        FormatKind::Json => {
            serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
        }

        #[cfg(feature = "yaml")]
        FormatKind::Yaml => {
            serde_yaml::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
        }

        #[cfg(feature = "csv")]
        FormatKind::Csv => deserialize_csv(bytes),

        #[cfg(feature = "xml")]
        FormatKind::Xml => {
            let s = String::from_utf8_lossy(bytes);
            quick_xml::de::from_str(&s).map_err(|e| FormatError::Serde(Box::new(e)))
        }

        #[cfg(feature = "markdown")]
        FormatKind::Markdown => deserialize_markdown(bytes),

        #[cfg(feature = "plaintext")]
        FormatKind::Plaintext => deserialize_plaintext(bytes),

        #[allow(unreachable_patterns)]
        _ => Err(FormatError::NotEnabled(kind)),
    }
}

/// Serialize to bytes using the specified format.
pub fn serialize<T: Serialize>(kind: FormatKind, value: &T) -> Result<Vec<u8>, FormatError> {
    match kind {
        #[cfg(feature = "json")]
        FormatKind::Json => {
            serde_json::to_vec_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))
        }

        #[cfg(feature = "yaml")]
        FormatKind::Yaml => serde_yaml::to_string(value)
            .map(|s| s.into_bytes())
            .map_err(|e| FormatError::Serde(Box::new(e))),

        #[cfg(feature = "csv")]
        FormatKind::Csv => serialize_csv(value),

        #[cfg(feature = "xml")]
        FormatKind::Xml => quick_xml::se::to_string(value)
            .map(|s| s.into_bytes())
            .map_err(|e| FormatError::Serde(Box::new(e))),

        #[cfg(feature = "markdown")]
        FormatKind::Markdown => serialize_markdown(value),

        #[cfg(feature = "plaintext")]
        FormatKind::Plaintext => serialize_plaintext(value),

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

/// Serialize to a writer using the specified format.
pub fn serialize_to_writer<T: Serialize>(
    kind: FormatKind,
    value: &T,
    writer: &mut dyn Write,
) -> Result<(), FormatError> {
    let bytes = serialize(kind, value)?;
    writer.write_all(&bytes)?;
    Ok(())
}

// === CSV implementation ===
#[cfg(feature = "csv")]
fn deserialize_csv<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);

    let records: Vec<csv::StringRecord> = rdr
        .records()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| FormatError::Serde(Box::new(e)))?;

    let headers = rdr.headers().map_err(|e| FormatError::Serde(Box::new(e)))?;
    let headers: Vec<&str> = headers.iter().collect();

    let json_records: Vec<serde_json::Value> = records
        .iter()
        .map(|record| {
            let mut obj = serde_json::Map::new();
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    obj.insert(
                        (*header).to_string(),
                        serde_json::Value::String(field.to_string()),
                    );
                }
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let json_value = serde_json::Value::Array(json_records);
    serde_json::from_value(json_value).map_err(|e| FormatError::Serde(Box::new(e)))
}

#[cfg(feature = "csv")]
fn serialize_csv<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    let json_value = serde_json::to_value(value).map_err(|e| FormatError::Serde(Box::new(e)))?;

    let mut wtr = csv::Writer::from_writer(Vec::new());

    match json_value {
        serde_json::Value::Array(arr) => {
            if let Some(first) = arr.first() {
                if let serde_json::Value::Object(obj) = first {
                    let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                    wtr.write_record(&headers)
                        .map_err(|e| FormatError::Serde(Box::new(e)))?;
                }
            }

            for item in arr {
                if let serde_json::Value::Object(obj) = item {
                    let record: Vec<String> = obj
                        .values()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .collect();
                    wtr.write_record(&record)
                        .map_err(|e| FormatError::Serde(Box::new(e)))?;
                }
            }
        }
        serde_json::Value::Object(obj) => {
            let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
            wtr.write_record(&headers)
                .map_err(|e| FormatError::Serde(Box::new(e)))?;

            let record: Vec<String> = obj
                .values()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect();
            wtr.write_record(&record)
                .map_err(|e| FormatError::Serde(Box::new(e)))?;
        }
        _ => {
            return Err(FormatError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "CSV format requires an array or object",
            ))));
        }
    }

    wtr.into_inner()
        .map_err(|e| FormatError::Other(Box::new(e)))
}

// === Plaintext implementation ===
#[cfg(feature = "plaintext")]
fn deserialize_plaintext<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let s = String::from_utf8_lossy(bytes);

    // Try JSON first if available
    #[cfg(feature = "json")]
    if let Ok(v) = serde_json::from_str(&s) {
        return Ok(v);
    }

    // Fall back to string deserializer
    let deserializer =
        serde::de::value::StringDeserializer::<serde::de::value::Error>::new(s.into_owned());
    T::deserialize(deserializer).map_err(|e| FormatError::Serde(Box::new(e)))
}

#[cfg(feature = "plaintext")]
fn serialize_plaintext<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    #[cfg(feature = "json")]
    {
        serde_json::to_vec_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))
    }
    #[cfg(not(feature = "json"))]
    {
        // Use debug format as fallback
        let _ = value; // suppress unused warning
        Err(FormatError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Plaintext serialization requires JSON feature",
        ))))
    }
}

// === Markdown implementation ===
#[cfg(feature = "markdown")]
fn extract_code_block(content: &str, lang: &str) -> Option<String> {
    let fence_start = format!("```{}", lang);
    let fence_end = "```";

    let start_idx = content.find(&fence_start)?;
    let content_start = start_idx + fence_start.len();
    let remaining = &content[content_start..];

    let content_start = if remaining.starts_with('\n') {
        content_start + 1
    } else {
        content_start
    };

    let remaining = &content[content_start..];
    let end_idx = remaining.find(fence_end)?;

    Some(remaining[..end_idx].trim_end().to_string())
}

#[cfg(feature = "markdown")]
fn deserialize_markdown<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let content = String::from_utf8_lossy(bytes);

    #[cfg(feature = "json")]
    if let Some(json_content) = extract_code_block(&content, "json") {
        return serde_json::from_str(&json_content).map_err(|e| FormatError::Serde(Box::new(e)));
    }

    #[cfg(feature = "yaml")]
    if let Some(yaml_content) = extract_code_block(&content, "yaml") {
        return serde_yaml::from_str(&yaml_content).map_err(|e| FormatError::Serde(Box::new(e)));
    }

    let deserializer =
        serde::de::value::StringDeserializer::<serde::de::value::Error>::new(content.into_owned());
    T::deserialize(deserializer).map_err(|e| FormatError::Serde(Box::new(e)))
}

#[cfg(feature = "markdown")]
fn serialize_markdown<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    #[cfg(feature = "json")]
    {
        let json_str =
            serde_json::to_string_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
        Ok(format!("```json\n{}\n```", json_str).into_bytes())
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = value;
        Err(FormatError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Markdown serialization requires JSON feature",
        ))))
    }
}

/// Registry for managing available formats, including custom formats.
#[derive(Debug, Clone, Default)]
pub struct FormatRegistry {
    /// Built-in format kinds
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
                return Some(kind.clone());
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
                return Ok(k.clone());
            }
            return Err(FormatError::UnknownFormat(k.clone()));
        }
        for k in candidates {
            if self.has_format(k) && k.is_available() {
                return Ok(k.clone());
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
                .ok_or_else(|| FormatError::UnknownFormat(kind.clone()))?;
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
                .ok_or_else(|| FormatError::UnknownFormat(kind.clone()))?;
            return custom.serialize(value);
        }

        // Handle built-in formats
        serialize(kind, value)
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
