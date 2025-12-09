//! Markdown format implementation.
//!
//! This is a simple implementation that treats markdown as plaintext with
//! optional code block extraction for structured data.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::{Format, FormatError, FormatKind};

/// Markdown format implementation.
///
/// For deserialization, it extracts content from code blocks if present,
/// otherwise treats the entire content as text.
/// For serialization, it wraps structured data in code blocks.
#[derive(Debug, Clone, Copy, Default)]
pub struct MarkdownFormat;

impl Format for MarkdownFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Markdown
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["md", "markdown"]
    }

    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        // Try to extract JSON from code blocks
        if let Some(json_content) = extract_code_block(&content, "json") {
            #[cfg(feature = "json")]
            return serde_json::from_str(&json_content)
                .map_err(|e| FormatError::Serde(Box::new(e)));
        }

        // Try to extract YAML from code blocks
        if let Some(yaml_content) = extract_code_block(&content, "yaml") {
            #[cfg(feature = "yaml")]
            return serde_yaml::from_str(&yaml_content)
                .map_err(|e| FormatError::Serde(Box::new(e)));
        }

        // Fallback: try to deserialize the raw content
        let deserializer =
            serde::de::value::StringDeserializer::<serde::de::value::Error>::new(content.clone());
        T::deserialize(deserializer).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    fn serialize<T: Serialize>(
        &self,
        value: &T,
        writer: &mut dyn Write,
    ) -> Result<(), FormatError> {
        // Serialize as JSON inside a code block
        #[cfg(feature = "json")]
        {
            let json_str =
                serde_json::to_string_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
            write!(writer, "```json\n{}\n```", json_str)?;
        }
        #[cfg(not(feature = "json"))]
        {
            let debug_str = format!("{:?}", value);
            write!(writer, "```\n{}\n```", debug_str)?;
        }
        Ok(())
    }
}

/// Extract content from a fenced code block with the given language.
fn extract_code_block(content: &str, lang: &str) -> Option<String> {
    let fence_start = format!("```{}", lang);
    let fence_end = "```";

    let start_idx = content.find(&fence_start)?;
    let content_start = start_idx + fence_start.len();
    let remaining = &content[content_start..];

    // Skip the newline after the opening fence
    let content_start = if remaining.starts_with('\n') {
        content_start + 1
    } else {
        content_start
    };

    let remaining = &content[content_start..];
    let end_idx = remaining.find(fence_end)?;

    Some(remaining[..end_idx].trim_end().to_string())
}
