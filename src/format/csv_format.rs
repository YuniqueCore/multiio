//! CSV format implementation.

use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::{Format, FormatError, FormatKind};

/// CSV format implementation using the csv crate.
#[derive(Debug, Clone, Copy, Default)]
pub struct CsvFormat;

impl Format for CsvFormat {
    fn kind(&self) -> FormatKind {
        FormatKind::Csv
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["csv"]
    }

    fn deserialize<T: DeserializeOwned>(&self, reader: &mut dyn Read) -> Result<T, FormatError> {
        // Read all content first
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        // Try to deserialize as a sequence of records
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        // Collect all records
        let records: Vec<csv::StringRecord> = rdr
            .records()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| FormatError::Serde(Box::new(e)))?;

        // Convert to JSON array format for serde deserialization
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

    fn serialize<T: Serialize>(
        &self,
        value: &T,
        writer: &mut dyn Write,
    ) -> Result<(), FormatError> {
        // Convert value to JSON first to handle nested structures
        let json_value =
            serde_json::to_value(value).map_err(|e| FormatError::Serde(Box::new(e)))?;

        let mut wtr = csv::Writer::from_writer(writer);

        match json_value {
            serde_json::Value::Array(arr) => {
                if let Some(first) = arr.first() {
                    // Write headers from first record's keys
                    if let serde_json::Value::Object(obj) = first {
                        let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                        wtr.write_record(&headers)
                            .map_err(|e| FormatError::Serde(Box::new(e)))?;
                    }
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
                // Single object: write as headers + single row
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

        wtr.flush().map_err(|e| FormatError::Io(e))
    }
}
