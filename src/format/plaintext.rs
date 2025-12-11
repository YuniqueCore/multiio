use std::io::Read;

use serde::{Serialize, de::DeserializeOwned};

use super::{FormatError, FormatKind, STRUCTURED_TEXT_FORMATS};

fn decode_from_string<T: DeserializeOwned>(s: String) -> Result<T, FormatError> {
    if let Some(v) = try_decode_structured::<T>(&s)? {
        return Ok(v);
    }

    // Fall back to string deserializer
    let deserializer = serde::de::value::StringDeserializer::<serde::de::value::Error>::new(s);
    T::deserialize(deserializer).map_err(|e| FormatError::Serde(Box::new(e)))
}

fn try_decode_structured<T: DeserializeOwned>(s: &str) -> Result<Option<T>, FormatError> {
    for kind in STRUCTURED_TEXT_FORMATS {
        match kind {
            FormatKind::Json => {
                #[cfg(feature = "json")]
                {
                    if let Ok(v) = serde_json::from_str(s) {
                        return Ok(Some(v));
                    }
                }
            }
            FormatKind::Yaml => {
                #[cfg(feature = "yaml")]
                {
                    if let Ok(v) = serde_yaml::from_str(s) {
                        return Ok(Some(v));
                    }
                }
            }
            FormatKind::Toml => {
                #[cfg(feature = "toml")]
                {
                    if let Ok(v) = toml::from_str(s) {
                        return Ok(Some(v));
                    }
                }
            }
            FormatKind::Ini => {
                #[cfg(feature = "ini")]
                {
                    if let Ok(v) = serde_ini::from_str(s) {
                        return Ok(Some(v));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let s = String::from_utf8_lossy(bytes).into_owned();
    decode_from_string(s)
}

pub(crate) fn stream_deserialize<T, R>(reader: R) -> impl Iterator<Item = Result<T, FormatError>>
where
    T: DeserializeOwned,
    R: Read,
{
    use std::io::{BufRead, BufReader};

    let buf = BufReader::new(reader);
    buf.lines().map(|res| match res {
        Ok(line) => decode_from_string(line),
        Err(e) => Err(FormatError::Io(e)),
    })
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
