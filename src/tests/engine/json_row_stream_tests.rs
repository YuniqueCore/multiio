use std::sync::Arc;

use crate::config::InputSpec;
use crate::format::FormatKind;
use crate::io::InMemorySource;
use crate::{ErrorPolicy, IoEngine, default_registry};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Row {
    name: String,
    value: i32,
}

fn make_json_engine(json: &str) -> IoEngine {
    let registry = default_registry();

    let src = Arc::new(InMemorySource::from_string("json", json));
    let spec = InputSpec::new("json", src)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    IoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], Vec::new())
}

#[test]
fn json_row_stream_reads_all_records() {
    let json = "{\"name\":\"foo\",\"value\":1}\n{\"name\":\"bar\",\"value\":2}\n";
    let engine = make_json_engine(json);

    let rows: Vec<Row> = engine
        .read_json_records::<Row>()
        .collect::<Result<_, _>>()
        .expect("json rows should parse");

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

#[test]
fn json_row_stream_reports_errors_for_non_json_format() {
    let registry = default_registry();
    let src = Arc::new(InMemorySource::from_string("csv", "name,value\nfoo,1\n"));
    let spec = InputSpec::new("csv", src)
        .with_format(FormatKind::Csv)
        .with_candidates(vec![FormatKind::Csv]);

    let engine = IoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], Vec::new());

    let mut iter = engine.read_json_records::<Row>();
    let err = iter
        .next()
        .expect("one result")
        .expect_err("expected error for non-json input");

    assert_eq!(err.stage, crate::error::Stage::ResolveInput);
    assert_eq!(err.target, "csv");
}
