//! JSON format implementation.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::{Format, FormatError, FormatKind};

/// JSON format implementation using serde_json.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonFormat;

impl Format for JsonFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Json
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["json"]
    }

    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError> {
        serde_json::from_reader(reader).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    fn serialize<T: Serialize>(
        &self,
        value: &T,
        writer: &mut dyn Write,
    ) -> Result<(), FormatError> {
        serde_json::to_writer_pretty(writer, value).map_err(|e| FormatError::Serde(Box::new(e)))
    }
}
