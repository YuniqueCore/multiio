//! Tests for ErrorPolicy behavior.

use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};

#[test]
fn error_policy_default_is_accumulate() {
    let policy = ErrorPolicy::default();
    assert_eq!(policy, ErrorPolicy::Accumulate);
}

#[test]
fn aggregate_error_single_and_len() {
    let err = SingleIoError {
        stage: Stage::Open,
        target: "test".to_string(),
        error: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "oops")),
    };

    let agg = AggregateError::single(err);
    assert_eq!(agg.len(), 1);
    assert!(!agg.is_empty());
}

#[test]
fn aggregate_error_from_single() {
    let err = SingleIoError {
        stage: Stage::Parse,
        target: "input".to_string(),
        error: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad")),
    };

    let agg: AggregateError = err.into();
    assert_eq!(agg.len(), 1);
}
