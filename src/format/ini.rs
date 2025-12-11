use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let s = String::from_utf8_lossy(bytes);
    serde_ini::from_str(&s).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    serde_ini::to_string(value)
        .map(|s| s.into_bytes())
        .map_err(|e| FormatError::Serde(Box::new(e)))
}
