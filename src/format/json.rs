//! JSON format implementation.

use serde::{Serialize, de::DeserializeOwned};
use std::io::Read;

use super::FormatError;

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    serde_json::to_vec_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn stream_deserialize<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read,
{
    serde_json::Deserializer::from_reader(reader)
        .into_iter::<T>()
        .map(|res| res.map_err(|e| FormatError::Serde(Box::new(e))))
}
