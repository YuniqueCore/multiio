//! YAML format implementation.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::{Format, FormatError, FormatKind};

/// YAML format implementation using serde_yaml.
#[derive(Debug, Clone, Copy, Default)]
pub struct YamlFormat;

impl Format for YamlFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Yaml
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["yaml", "yml"]
    }

    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError> {
        serde_yaml::from_reader(reader).map_err(|e| FormatError::Serde(Box::new(e)))
    }

    fn serialize<T: Serialize>(
        &self,
        value: &T,
        writer: &mut dyn Write,
    ) -> Result<(), FormatError> {
        serde_yaml::to_writer(writer, value).map_err(|e| FormatError::Serde(Box::new(e)))
    }
}
