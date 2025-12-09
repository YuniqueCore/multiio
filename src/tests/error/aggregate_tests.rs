//! Tests for AggregateError formatting.

use crate::error::{AggregateError, SingleIoError, Stage};

#[test]
fn aggregate_error_display_includes_count() {
    let e1 = SingleIoError {
        stage: Stage::Open,
        target: "a".to_string(),
        error: Box::new(std::io::Error::other("e1")),
    };
    let e2 = SingleIoError {
        stage: Stage::Parse,
        target: "b".to_string(),
        error: Box::new(std::io::Error::other("e2")),
    };

    let agg = AggregateError {
        errors: vec![e1, e2],
    };

    let s = format!("{}", agg);
    assert!(s.contains("2 error(s)"));
}
