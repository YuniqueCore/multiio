use std::io::Cursor;

use crate::format::{CustomFormat, FormatError, FormatKind, FormatRegistry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Row {
    name: String,
    value: i32,
}

#[test]
fn custom_streaming_deserialize_via_registry() {
    let mut registry = FormatRegistry::new();

    // Define an NDJSON-like custom format with streaming support
    let fmt = CustomFormat::new("ndjson", &["ndjson"])
        .with_deserialize(|bytes| {
            // Fallback non-streaming handler: parse a single JSON value
            serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_serialize(|value| {
            serde_json::to_vec(value).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_stream_deserialize(|reader| {
            use std::io::{BufRead, BufReader};

            let buf = BufReader::new(reader);
            let iter = buf.lines().map(|line_res| {
                let line = line_res.map_err(FormatError::Io)?;
                let value: serde_json::Value =
                    serde_json::from_str(&line).map_err(|e| FormatError::Serde(Box::new(e)))?;
                Ok(value)
            });

            Box::new(iter) as Box<dyn Iterator<Item = Result<serde_json::Value, FormatError>>>
        });

    registry.register_custom(fmt);

    let input = "{\"name\":\"foo\",\"value\":1}\n{\"name\":\"bar\",\"value\":2}\n";
    let reader = Cursor::new(input.as_bytes());

    let iter = registry
        .stream_deserialize_into::<Row>(Some(&FormatKind::Custom("ndjson")), &[], Box::new(reader))
        .expect("streaming should be supported");

    let rows: Vec<Row> = iter.collect::<Result<_, _>>().expect("rows should parse");

    assert_eq!(
        rows,
        vec![
            Row {
                name: "foo".into(),
                value: 1,
            },
            Row {
                name: "bar".into(),
                value: 2,
            },
        ]
    );
}
