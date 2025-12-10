//! Custom format tests.

use crate::format::{CustomFormat, FormatError, FormatKind, FormatRegistry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
}

#[test]
fn test_custom_format_registration() {
    let mut registry = FormatRegistry::new();

    // Register a custom format
    let custom = CustomFormat::new("test-format", &["tf"])
        .with_deserialize(|bytes| {
            serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_serialize(|value| {
            serde_json::to_vec(value).map_err(|e| FormatError::Serde(Box::new(e)))
        });

    registry.register_custom(custom);

    // Check registration
    assert!(registry.has_format(&FormatKind::Custom("test-format")));
    assert!(registry.get_custom("test-format").is_some());
}

#[test]
fn test_custom_format_extension_lookup() {
    let mut registry = FormatRegistry::new();

    let custom = CustomFormat::new("my-format", &["myf", "myfmt"])
        .with_deserialize(|bytes| {
            serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_serialize(|value| {
            serde_json::to_vec(value).map_err(|e| FormatError::Serde(Box::new(e)))
        });

    registry.register_custom(custom);

    // Test extension lookup
    let kind = registry.kind_for_extension("myf");
    assert_eq!(kind, Some(FormatKind::Custom("my-format")));

    let kind = registry.kind_for_extension("myfmt");
    assert_eq!(kind, Some(FormatKind::Custom("my-format")));
}

#[test]
fn test_custom_format_serialize_deserialize() {
    let mut registry = FormatRegistry::new();

    // Create a custom format that wraps data in brackets
    let bracket_format = CustomFormat::new("bracket", &["brk"])
        .with_deserialize(|bytes| {
            // Remove brackets and parse as JSON
            let s = String::from_utf8_lossy(bytes);
            let inner = s.trim_start_matches('[').trim_end_matches(']');
            serde_json::from_str(inner).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_serialize(|value| {
            let json = serde_json::to_string(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
            Ok(format!("[{}]", json).into_bytes())
        });

    registry.register_custom(bracket_format);

    // Test serialization
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };

    let bytes = registry
        .serialize_value(Some(&FormatKind::Custom("bracket")), &[], &data)
        .unwrap();

    let output = String::from_utf8(bytes).unwrap();
    assert!(output.starts_with('['));
    assert!(output.ends_with(']'));

    // Test deserialization
    let result: TestData = registry
        .deserialize_value(Some(&FormatKind::Custom("bracket")), &[], output.as_bytes())
        .unwrap();

    assert_eq!(result, data);
}

#[test]
fn custom_format_without_deserialize_errors() {
    let fmt = CustomFormat::new("no-deser", &["and"]).with_serialize(|value| {
        serde_json::to_vec(value).map_err(|e| FormatError::Serde(Box::new(e)))
    });

    let err = fmt
        .deserialize::<serde_json::Value>(b"{}")
        .expect_err("expected error when deserializing without handler");

    match err {
        FormatError::Other(inner) => {
            let msg = inner.to_string();
            assert!(
                msg.contains("does not support deserialization"),
                "unexpected message: {}",
                msg
            );
        }
        other => panic!("expected FormatError::Other, got: {other:?}"),
    }
}

#[test]
fn custom_format_without_serialize_errors() {
    let fmt = CustomFormat::new("no-serial", &["ns"]).with_deserialize(|bytes| {
        serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
    });

    let value = TestData {
        name: "x".to_string(),
        value: 1,
    };

    let err = fmt
        .serialize(&value)
        .expect_err("expected error when serializing without handler");

    match err {
        FormatError::Other(inner) => {
            let msg = inner.to_string();
            assert!(
                msg.contains("does not support serialization"),
                "unexpected message: {}",
                msg
            );
        }
        other => panic!("expected FormatError::Other, got: {other:?}"),
    }
}
