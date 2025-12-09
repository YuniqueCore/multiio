//! CSV format implementation.

use serde::{Serialize, de::DeserializeOwned};

use super::FormatError;

pub(crate) fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FormatError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);

    let records: Vec<csv::StringRecord> = rdr
        .records()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| FormatError::Serde(Box::new(e)))?;

    let headers = rdr.headers().map_err(|e| FormatError::Serde(Box::new(e)))?;
    let headers: Vec<&str> = headers.iter().collect();

    let json_records: Vec<serde_json::Value> = records
        .iter()
        .map(|record| {
            let mut obj = serde_json::Map::new();
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    obj.insert(
                        (*header).to_string(),
                        serde_json::Value::String(field.to_string()),
                    );
                }
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let json_value = serde_json::Value::Array(json_records);
    serde_json::from_value(json_value).map_err(|e| FormatError::Serde(Box::new(e)))
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, FormatError> {
    let json_value = serde_json::to_value(value).map_err(|e| FormatError::Serde(Box::new(e)))?;

    let mut wtr = csv::Writer::from_writer(Vec::new());

    match json_value {
        serde_json::Value::Array(arr) => {
            if let Some(first) = arr.first()
                && let serde_json::Value::Object(obj) = first
            {
                let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                wtr.write_record(&headers)
                    .map_err(|e| FormatError::Serde(Box::new(e)))?;
            }

            for item in arr {
                if let serde_json::Value::Object(obj) = item {
                    let record: Vec<String> = obj
                        .values()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .collect();
                    wtr.write_record(&record)
                        .map_err(|e| FormatError::Serde(Box::new(e)))?;
                }
            }
        }
        serde_json::Value::Object(obj) => {
            let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
            wtr.write_record(&headers)
                .map_err(|e| FormatError::Serde(Box::new(e)))?;

            let record: Vec<String> = obj
                .values()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect();
            wtr.write_record(&record)
                .map_err(|e| FormatError::Serde(Box::new(e)))?;
        }
        _ => {
            return Err(FormatError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "CSV format requires an array or object",
            ))));
        }
    }

    wtr.into_inner()
        .map_err(|e| FormatError::Other(Box::new(e)))
}
