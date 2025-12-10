//! YAML format implementation.

use std::io::Read;

use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    serde_yaml::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn stream_deserialize<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read + 'static,
{
    serde_yaml::Deserializer::from_reader(reader)
        .map(|doc| T::deserialize(doc).map_err(|e| FormatError::Serde(Box::new(e))))
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    serde_yaml::to_string(value)
        .map(|s| s.into_bytes())
        .map_err(|e| FormatError::Serde(Box::new(e)))
}
