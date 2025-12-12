//! Format abstraction for serialization and deserialization.
//!
//! This module provides:
//! - `FormatKind`: Enum representing different data formats
//! - `FormatError`: Errors that can occur during format operations
//! - `FormatRegistry`: Registry managing formats by kind
//! - `CustomFormat`: Support for user-defined custom formats

use std::io::Read;

use paste::paste;

mod custom;
pub use custom::CustomFormat;

// Per-format implementations
#[cfg(feature = "csv")]
mod csv;
#[cfg(feature = "ini")]
mod ini;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "markdown")]
mod markdown;
#[cfg(feature = "plaintext")]
mod plaintext;
#[cfg(feature = "toml")]
mod toml;
#[cfg(feature = "xml")]
mod xml;
#[cfg(feature = "yaml")]
mod yaml;

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FormatKind {
    Json,
    Yaml,
    Toml,
    Csv,
    Xml,
    Ini,
    Markdown,
    /// Custom format with a unique name
    Custom(&'static str),
    Plaintext,
}

impl Copy for FormatKind {}

/// Central spec for all builtin (non-custom) formats.
///
/// Fields: (Category, Variant, feature, module, display, extensions, aliases)
macro_rules! format_spec {
    // Allow passing extra arguments through to the projection macro.
    ($mac:ident ( $($args:tt)* )) => {
        $mac! {
            $($args)*
            (Structured, Json,      "json",      json,      "json",      ["json"],            ["json"])
            (Structured, Yaml,      "yaml",      yaml,      "yaml",      ["yaml", "yml"],    ["yaml", "yml"])
            (Structured, Toml,      "toml",      toml,      "toml",      ["toml"],            ["toml"])
            (Structured, Ini,       "ini",       ini,       "ini",       ["ini"],             ["ini"])
            (Other,      Csv,       "csv",       csv,       "csv",       ["csv"],             ["csv"])
            (Other,      Xml,       "xml",       xml,       "xml",       ["xml"],             ["xml"])
            (Other,      Markdown,  "markdown",  markdown,  "markdown",  ["md", "markdown"], ["markdown", "md"])
            (Other,      Plaintext, "plaintext", plaintext, "plaintext", ["txt", "text"],    ["plaintext", "text", "txt"])
        }
    };

    ($mac:ident) => {
        $mac! {
            (Structured, Json,      "json",      json,      "json",      ["json"],            ["json"])
            (Structured, Yaml,      "yaml",      yaml,      "yaml",      ["yaml", "yml"],    ["yaml", "yml"])
            (Structured, Toml,      "toml",      toml,      "toml",      ["toml"],            ["toml"])
            (Structured, Ini,       "ini",       ini,       "ini",       ["ini"],             ["ini"])
            (Other,      Csv,       "csv",       csv,       "csv",       ["csv"],             ["csv"])
            (Other,      Xml,       "xml",       xml,       "xml",       ["xml"],             ["xml"])
            (Other,      Markdown,  "markdown",  markdown,  "markdown",  ["md", "markdown"], ["markdown", "md"])
            (Other,      Plaintext, "plaintext", plaintext, "plaintext", ["txt", "text"],    ["plaintext", "text", "txt"])
        }
    };
}

// Projection: full `Display` implementation for `FormatKind`.
macro_rules! impl_formatkind_display {
    ( $(($cat:ident, $kind:ident, $feat:literal, $module:ident,
        $display:literal, [$($ext:literal),*], [$($alias:literal),*]))* ) => {
        impl std::fmt::Display for FormatKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( FormatKind::$kind => write!(f, $display), )*
                    FormatKind::Custom(name) => write!(f, "{}", name),
                }
            }
        }
    };
}

// Projection: default order = all kinds in declaration order.
macro_rules! define_default_order_from_spec {
    ( $(($cat:ident, $kind:ident, $feat:literal, $module:ident,
        $display:literal, [$($ext:literal),*], [$($alias:literal),*]))* ) => {
        pub(crate) const DEFAULT_FORMAT_ORDER: &[FormatKind] = &[
            $( FormatKind::$kind ),*
        ];
    };
}

// Projection: structured-text formats = only `Structured` entries, same order.
// NOTE: This pattern assumes that all `Structured` entries appear before `Other`
// entries in `format_spec!`. Tests assert that structured formats are a prefix
// of `DEFAULT_FORMAT_ORDER`, so reordering must respect this invariant.
macro_rules! define_structured_text_from_spec {
    (
        $(
            (Structured, $kind:ident, $feat:literal, $module:ident,
             $display:literal, [$($ext:literal),*], [$($alias:literal),*])
        )*
        $(
            (Other, $other_kind:ident, $other_feat:literal, $other_module:ident,
             $other_display:literal, [$($other_ext:literal),*], [$($other_alias:literal),*])
        )*
    ) => {
        pub(crate) const STRUCTURED_TEXT_FORMATS: &[FormatKind] = &[
            $( FormatKind::$kind, )*
        ];
    };
}

format_spec!(define_default_order_from_spec);
format_spec!(define_structured_text_from_spec);
format_spec!(impl_formatkind_display);

// Helper: iterate over all enabled builtin formats in DEFAULT_FORMAT_ORDER.
macro_rules! impl_for_each_enabled_builtin {
    ( $(($cat:ident, $kind:ident, $feat:literal, $module:ident,
        $display:literal, [$($ext:literal),*], [$($alias:literal),*]))* ) => {
        pub(crate) fn for_each_enabled_builtin<F>(mut f: F)
        where
            F: FnMut(FormatKind),
        {
            for kind in DEFAULT_FORMAT_ORDER {
                match kind {
                    $(
                        FormatKind::$kind => {
                            #[cfg(feature = $feat)]
                            f(FormatKind::$kind);
                        }
                    )*
                    FormatKind::Custom(_) => {}
                }
            }
        }
    };
}

format_spec!(impl_for_each_enabled_builtin);

// Projection: body for `FormatKind::extensions`.
macro_rules! impl_formatkind_extensions_body {
    ($self:ident
        $(($cat:ident, $kind:ident, $feat:literal, $module:ident,
           $display:literal, [$($ext:literal),*], [$($alias:literal),*]))*
    ) => {{
        match $self {
            $( FormatKind::$kind => &[$($ext),*], )*
            FormatKind::Custom(_) => &[],
        }
    }};
}

// Projection: body for `FormatKind::is_available`.
macro_rules! impl_formatkind_is_available_body {
    ($self:ident
        $(($cat:ident, $kind:ident, $feat:literal, $module:ident,
           $display:literal, [$($ext:literal),*], [$($alias:literal),*]))*
    ) => {{
        match $self {
            $(
                #[cfg(feature = $feat)]
                FormatKind::$kind => true,
                #[cfg(not(feature = $feat))]
                FormatKind::$kind => false,
            )*
            // Custom formats are always considered available
            // (availability is determined by registration)
            FormatKind::Custom(_) => true,
        }
    }};
}

// Projection: body for `FromStr` implementation.
macro_rules! impl_formatkind_from_str_body {
    ($lower:ident
        $(($cat:ident, $kind:ident, $feat:literal, $module:ident,
           $display:literal, [$($ext:literal),*], [$($alias:literal),*]))*
    ) => {{
        let kind = match $lower.as_str() {
            $(
                $( $alias )|* => FormatKind::$kind,
            )*
            _ => return Err(()),
        };
        Ok(kind)
    }};
}

impl FormatKind {
    pub fn custom(name: &'static str) -> Self {
        FormatKind::Custom(name)
    }

    /// Get file extensions for this format.
    /// Note: For custom formats, this returns an empty slice.
    /// Use FormatRegistry to get extensions for custom formats.
    pub fn extensions(&self) -> &'static [&'static str] {
        format_spec!(impl_formatkind_extensions_body(self))
    }

    /// Check if this format is available (feature enabled).
    pub fn is_available(&self) -> bool {
        format_spec!(impl_formatkind_is_available_body(self))
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

        format_spec!(impl_formatkind_from_str_body(lower))
    }
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Unknown format: {0}")]
    UnknownFormat(FormatKind),

    #[error("No format matched the input")]
    NoFormatMatched,

    #[error("Format '{0}' is not enabled. Enable the corresponding feature.")]
    NotEnabled(FormatKind),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde error: {0}")]
    Serde(Box<dyn std::error::Error + Send + Sync>),

    /// Other format-specific error
    #[error("Format error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

// Projection: body for top-level `deserialize` function.
macro_rules! impl_deserialize_body {
    ($bytes:ident, $kind:ident
        $(($cat:ident, $fmt_kind:ident, $feat:literal, $module:ident,
           $display:literal, [$($ext:literal),*], [$($alias:literal),*]))*
    ) => {{
        match $kind {
            $(
                #[cfg(feature = $feat)]
                FormatKind::$fmt_kind => $module::deserialize($bytes),
            )*

            #[allow(unreachable_patterns)]
            _ => Err(FormatError::NotEnabled($kind)),
        }
    }};
}

// Projection: body for top-level `serialize` function.
macro_rules! impl_serialize_body {
    ($value:ident, $kind:ident
        $(($cat:ident, $fmt_kind:ident, $feat:literal, $module:ident,
           $display:literal, [$($ext:literal),*], [$($alias:literal),*]))*
    ) => {{
        match $kind {
            $(
                #[cfg(feature = $feat)]
                FormatKind::$fmt_kind => $module::serialize($value),
            )*

            #[allow(unreachable_patterns)]
            _ => Err(FormatError::NotEnabled($kind)),
        }
    }};
}

pub fn deserialize<T: DeserializeOwned>(kind: FormatKind, bytes: &[u8]) -> Result<T, FormatError> {
    format_spec!(impl_deserialize_body(bytes, kind))
}

/// Serialize to bytes using the specified format.
pub fn serialize<T: Serialize>(kind: FormatKind, value: &T) -> Result<Vec<u8>, FormatError> {
    format_spec!(impl_serialize_body(value, kind))
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

macro_rules! define_stream_deserialize_fn_read {
    (
        $(#[$meta:meta])*
        [$cfg_feat:literal]
        $module:ident
    ) => {
        paste! {
            $(#[$meta])*
            #[cfg(feature = $cfg_feat)]
            pub fn [<deserialize_ $module _stream>]<T, R>(
                reader: R,
            ) -> impl Iterator<Item = Result<T, FormatError>>
            where
                T: DeserializeOwned,
                R: Read,
            {
                $module::stream_deserialize(reader)
            }
        }
    };
}

macro_rules! define_stream_deserialize_fn_read_static {
    (
        $(#[$meta:meta])*
        [$cfg_feat:literal]
        $module:ident
    ) => {
        paste! {
            $(#[$meta])*
            #[cfg(feature = $cfg_feat)]
            pub fn [<deserialize_ $module _stream>]<T, R>(
                reader: R,
            ) -> impl Iterator<Item = Result<T, FormatError>>
            where
                T: DeserializeOwned,
                R: Read + 'static,
            {
                $module::stream_deserialize(reader)
            }
        }
    };
}

define_stream_deserialize_fn_read!(
    /// Stream JSON values from a reader as multiple top-level JSON documents.
    ["json"]
    json
);

define_stream_deserialize_fn_read!(
    /// Stream CSV records from a reader.
    ["csv"]
    csv
);

define_stream_deserialize_fn_read_static!(
    /// Stream YAML documents from a reader.
    ["yaml"]
    yaml
);

define_stream_deserialize_fn_read!(
    /// Stream plaintext records (typically lines) from a reader.
    ["plaintext"]
    plaintext
);

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
///
/// default formats with order: [DEFAULT_FORMAT_ORDER]
pub fn default_registry() -> FormatRegistry {
    let mut registry = FormatRegistry::new();
    for_each_enabled_builtin(|k| registry.register(k));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_format_order_is_expected() {
        assert_eq!(
            DEFAULT_FORMAT_ORDER,
            &[
                FormatKind::Json,
                FormatKind::Yaml,
                FormatKind::Toml,
                FormatKind::Ini,
                FormatKind::Csv,
                FormatKind::Xml,
                FormatKind::Markdown,
                FormatKind::Plaintext,
            ],
        );
    }

    #[test]
    fn structured_text_formats_are_prefix_of_default_order() {
        assert_eq!(
            STRUCTURED_TEXT_FORMATS,
            &DEFAULT_FORMAT_ORDER[..STRUCTURED_TEXT_FORMATS.len()],
        );
    }
}

// Async format support
#[cfg(feature = "async")]
mod async_format;

#[cfg(feature = "async")]
pub use async_format::*;
