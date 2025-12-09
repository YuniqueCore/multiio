//! JSON format tests.

use multiio::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
}

#[test]
fn test_json_roundtrip() {
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };

    // Serialize
    let bytes = serialize(FormatKind::Json, &data).unwrap();
    assert!(!bytes.is_empty());

    // Deserialize
    let result: TestData = deserialize(FormatKind::Json, &bytes).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_json_deserialize_string() {
    let json = r#"{"name": "hello", "value": 123}"#;
    let result: TestData = deserialize(FormatKind::Json, json.as_bytes()).unwrap();
    assert_eq!(result.name, "hello");
    assert_eq!(result.value, 123);
}
