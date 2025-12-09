//! JSON format implementation.

use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    serde_json::to_vec_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))
}
