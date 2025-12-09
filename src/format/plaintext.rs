//! Plaintext format implementation.

use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let s = String::from_utf8_lossy(bytes);

    // Try JSON first if available
    #[cfg(feature = "json")]
    if let Ok(v) = serde_json::from_str(&s) {
        return Ok(v);
    }

    // Fall back to string deserializer
    let deserializer =
        serde::de::value::StringDeserializer::<serde::de::value::Error>::new(s.into_owned());
    T::deserialize(deserializer).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    #[cfg(feature = "json")]
    {
        serde_json::to_vec_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = value;
        Err(FormatError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Plaintext serialization requires JSON feature",
        ))))
    }
}
