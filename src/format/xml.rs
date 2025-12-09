//! XML format implementation.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::{Format, FormatError, FormatKind};

/// XML format implementation using quick-xml.
#[derive(Debug, Clone, Copy, Default)]
pub struct XmlFormat;

impl Format for XmlFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Xml
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["xml"]
    }

    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        quick_xml::de::from_str(&content).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    fn serialize<T: Serialize>(
        &self,
        value: &T,
        writer: &mut dyn Write,
    ) -> Result<(), FormatError> {
        let xml_string =
            quick_xml::se::to_string(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
        writer.write_all(xml_string.as_bytes())?;
        Ok(())
    }
}
