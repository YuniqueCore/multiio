//! Tests for CLI helper functions (parse_format, infer_format_from_path).

use crate::FormatKind;
use crate::cli::{infer_format_from_path, parse_format};

#[test]
fn test_parse_format() {
    assert_eq!(parse_format("json"), Some(FormatKind::Json));
    assert_eq!(parse_format("JSON"), Some(FormatKind::Json));
    assert_eq!(parse_format("yaml"), Some(FormatKind::Yaml));
    assert_eq!(parse_format("yml"), Some(FormatKind::Yaml));
    assert_eq!(parse_format("unknown"), None);
}

#[test]
fn test_infer_format_from_path() {
    assert_eq!(
        infer_format_from_path("config.json"),
        Some(FormatKind::Json)
    );
    assert_eq!(infer_format_from_path("data.yaml"), Some(FormatKind::Yaml));
    assert_eq!(infer_format_from_path("file.unknown"), None);
}
