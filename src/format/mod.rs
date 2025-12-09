//! Format abstraction for serialization and deserialization.
//!
//! This module provides:
//! - `FormatKind`: Enum representing different data formats
//! - `FormatError`: Errors that can occur during format operations
//! - `Format`: Trait for synchronous format implementations
//! - `FormatRegistry`: Registry for managing format implementations

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

// Format implementations
#[cfg(feature = "csv")]
mod csv_format;
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

// Re-exports
#[cfg(feature = "csv")]
pub use csv_format::CsvFormat;
#[cfg(feature = "json")]
pub use json::JsonFormat;
#[cfg(feature = "markdown")]
pub use markdown::MarkdownFormat;
#[cfg(feature = "plaintext")]
pub use plaintext::PlaintextFormat;
#[cfg(feature = "xml")]
pub use xml::XmlFormat;
#[cfg(feature = "yaml")]
pub use yaml::YamlFormat;

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
    /// Custom format with a static name
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

/// Trait for synchronous format implementations.
///
/// Implementors provide serialization and deserialization capabilities
/// for a specific data format.
pub trait Format: Send + Sync + 'static {
    /// Returns the kind of this format.
    fn kind(&self) -> FormatKind;

    /// Returns the file extensions associated with this format.
    ///
    /// For example, `["json"]` for JSON, `["yml", "yaml"]` for YAML.
    fn extensions(&self) -> &'static [&'static str];

    /// Deserialize a value from a reader.
    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError>;

    /// Serialize a value to a writer.
    fn serialize<T: Serialize>(&self, value: &T, writer: &mut dyn Write)
    -> Result<(), FormatError>;
}

/// Registry for managing format implementations.
pub struct FormatRegistry {
    formats: Vec<Box<dyn Format>>,
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    /// Register a format implementation.
    pub fn register(&mut self, format: Box<dyn Format>) {
        self.formats.push(format);
    }

    /// Register a format implementation (builder pattern).
    pub fn with_format(mut self, format: Box<dyn Format>) -> Self {
        self.register(format);
        self
    }

    /// Get a format by its kind.
    pub fn format_for_kind(&self, kind: &FormatKind) -> Option<&dyn Format> {
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
    ///
    /// If `explicit` is `Some`, returns that format.
    /// Otherwise, tries each candidate in order.
    pub fn resolve(
        &self,
        explicit: Option<&FormatKind>,
        candidates: &[FormatKind],
    ) -> Result<&dyn Format, FormatError> {
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

    /// Get all registered formats.
    pub fn formats(&self) -> &[Box<dyn Format>] {
        &self.formats
    }
}

/// Create a default registry with all enabled formats.
pub fn default_registry() -> FormatRegistry {
    let mut registry = FormatRegistry::new();

    #[cfg(feature = "json")]
    registry.register(Box::new(JsonFormat));

    #[cfg(feature = "yaml")]
    registry.register(Box::new(YamlFormat));

    #[cfg(feature = "plaintext")]
    registry.register(Box::new(PlaintextFormat));

    #[cfg(feature = "csv")]
    registry.register(Box::new(CsvFormat));

    #[cfg(feature = "xml")]
    registry.register(Box::new(XmlFormat));

    #[cfg(feature = "markdown")]
    registry.register(Box::new(MarkdownFormat));

    registry
}

// Async format support
#[cfg(feature = "async")]
mod async_format;

#[cfg(feature = "async")]
pub use async_format::*;
