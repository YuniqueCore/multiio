//! End-to-end and error-policy tests for IoEngine.

use std::sync::Arc;

use crate::config::{FileExistsPolicy, InputSpec, OutputSpec};
use crate::error::{AggregateError, ErrorPolicy, Stage};
use crate::io::{InMemorySink, InMemorySource, InputProvider};
use crate::{FormatKind, IoEngine, default_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    name: String,
    value: i32,
}

fn make_engine(
    error_policy: ErrorPolicy,
    inputs: Vec<InputSpec>,
    outputs: Vec<OutputSpec>,
) -> IoEngine {
    let registry = default_registry();
    IoEngine::new(registry, error_policy, inputs, outputs)
}

#[test]
fn sync_engine_read_write_inmemory_ok() {
    // Prepare in-memory JSON input
    let json = r#"{"name": "a", "value": 1}"#;
    let src = Arc::new(InMemorySource::from_string("in", json));

    let input_spec = InputSpec::new("in", src)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    // Prepare in-memory output
    let sink = Arc::new(InMemorySink::new("out"));
    let output_spec = OutputSpec::new("out", sink.clone())
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json])
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    let engine = make_engine(ErrorPolicy::FastFail, vec![input_spec], vec![output_spec]);

    // End-to-end: read then write
    let values: Vec<Config> = engine.read_all().expect("read_all should succeed");
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].name, "a");
    assert_eq!(values[0].value, 1);

    engine.write_all(&values).expect("write_all should succeed");

    // Verify output JSON is valid and decodes back to same value
    let out_str = sink.contents_string();
    let decoded: Vec<Config> = serde_json::from_str(&out_str).expect("output must be valid json");
    assert_eq!(decoded, values);
}

#[test]
fn sync_engine_fast_fail_on_open_error() {
    // A fake input that always fails on open, simulating network/FS errors.
    #[derive(Debug)]
    struct FailingInput {
        id: String,
    }

    impl InputProvider for FailingInput {
        fn id(&self) -> &str {
            &self.id
        }

        fn open(&self) -> std::io::Result<Box<dyn std::io::Read + Send>> {
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "simulated network timeout",
            ))
        }
    }

    let src = Arc::new(FailingInput {
        id: "net://example".to_string(),
    });

    let input_spec = InputSpec::new("net://example", src)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    // A dummy output; should never be written in this test.
    let sink = Arc::new(InMemorySink::new("out"));
    let output_spec = OutputSpec::new("out", sink)
        .with_format(FormatKind::Json)
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    let engine = make_engine(ErrorPolicy::FastFail, vec![input_spec], vec![output_spec]);

    let result: Result<Vec<Config>, AggregateError> = engine.read_all();
    let err = result.expect_err("expected failure due to open error");

    assert_eq!(err.errors.len(), 1);
    let e = &err.errors[0];
    assert_eq!(e.stage, Stage::Open);
    assert_eq!(e.target, "net://example");
}

#[test]
fn sync_engine_accumulate_parse_errors() {
    // First input: valid JSON
    let json_ok = r#"{"name": "ok", "value": 1}"#;
    let src_ok = Arc::new(InMemorySource::from_string("ok", json_ok));
    let spec_ok = InputSpec::new("ok", src_ok)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    // Two invalid JSON inputs to trigger parse errors
    let src_bad1 = Arc::new(InMemorySource::from_string("bad1", "{not-json"));
    let spec_bad1 = InputSpec::new("bad1", src_bad1)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let src_bad2 = Arc::new(InMemorySource::from_string("bad2", "[1,2,,]"));
    let spec_bad2 = InputSpec::new("bad2", src_bad2)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let sink = Arc::new(InMemorySink::new("out"));
    let out_spec = OutputSpec::new("out", sink)
        .with_format(FormatKind::Json)
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    let engine = make_engine(
        ErrorPolicy::Accumulate,
        vec![spec_ok, spec_bad1, spec_bad2],
        vec![out_spec],
    );

    let result: Result<Vec<Config>, AggregateError> = engine.read_all();
    let agg = result.expect_err("expected aggregate error in accumulate mode");

    // We expect two parse errors (bad1, bad2)
    assert_eq!(agg.errors.len(), 2);
    assert!(agg.errors.iter().all(|e| e.stage == Stage::Parse));

    // Targets should include the failing specs
    let targets: Vec<_> = agg.errors.iter().map(|e| e.target.as_str()).collect();
    assert!(targets.contains(&"bad1"));
    assert!(targets.contains(&"bad2"));
}

// Async engine tests could be added here with cfg(feature = "async") if desired.
