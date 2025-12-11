use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

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

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
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

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
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
