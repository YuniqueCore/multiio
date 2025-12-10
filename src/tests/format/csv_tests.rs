#![cfg(feature = "csv")]

//! CSV format roundtrip tests.

use crate::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CsvRow {
    name: String,
    value: i32,
}

#[test]
fn csv_roundtrip_array_of_objects() {
    let rows = vec![
        CsvRow {
            name: "a".into(),
            value: 1,
        },
        CsvRow {
            name: "b".into(),
            value: 2,
        },
    ];

    let bytes = serialize(FormatKind::Csv, &rows).expect("serialize csv");
    assert!(!bytes.is_empty());

    let decoded: Vec<CsvRow> = deserialize(FormatKind::Csv, &bytes).expect("deserialize csv");
    assert_eq!(decoded, rows);
}

#[test]
fn csv_roundtrip_single_object() {
    let row = CsvRow {
        name: "single".into(),
        value: 10,
    };

    let bytes = serialize(FormatKind::Csv, &row).expect("serialize csv single");

    // CSV implementation always models data as an array of objects, even for a
    // single record, so we deserialize into Vec<CsvRow> and check the single
    // element.
    let decoded: Vec<CsvRow> =
        deserialize(FormatKind::Csv, &bytes).expect("deserialize csv single as array");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0], row);
}

#[test]
fn csv_errors_on_non_object() {
    let data = vec![1, 2, 3];
    let res = serialize(FormatKind::Csv, &data);
    assert!(res.is_err());
}
