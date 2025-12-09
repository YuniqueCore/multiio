//! Plaintext format implementation.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::{Format, FormatError, FormatKind};

/// Plaintext format implementation.
///
/// This format treats data as raw strings. For deserialization, it reads
/// the entire content as a string and attempts to deserialize from that.
/// For serialization, it converts the value to a string representation.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaintextFormat;

impl Format for PlaintextFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Plaintext
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["txt", "text"]
    }

    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        // Try to deserialize using serde's string deserializer
        // This works for types that implement FromStr or can be deserialized from a string
        let deserializer =
            serde::de::value::StringDeserializer::<serde::de::value::Error>::new(content.clone());
        T::deserialize(deserializer).map_err(|e| {
            // If string deserialization fails, try as JSON (for complex types)
            #[cfg(feature = "json")]
            if let Ok(val) = serde_json::from_str::<T>(&content) {
                return FormatError::Other(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Fallback succeeded",
                )));
            }
            FormatError::Serde(Box::new(e))
        })
    }

    fn serialize<T: Serialize>(
        &self,
        value: &T,
        writer: &mut dyn Write,
    ) -> Result<(), FormatError> {
        // Try to serialize as a string first
        #[cfg(feature = "json")]
        {
            let json_str =
                serde_json::to_string_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
            writer.write_all(json_str.as_bytes())?;
        }
        #[cfg(not(feature = "json"))]
        {
            // Fallback: use Debug formatting
            let debug_str = format!("{:?}", value);
            writer.write_all(debug_str.as_bytes())?;
        }
        Ok(())
    }
}
