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

fn make_csv_engine(csv: &str) -> IoEngine {
    let registry = default_registry();

    let src = Arc::new(InMemorySource::from_string("csv", csv));
    let spec = InputSpec::new("csv", src)
        .with_format(FormatKind::Csv)
        .with_candidates(vec![FormatKind::Csv]);

    IoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], Vec::new())
}

#[test]
fn csv_row_stream_reads_all_records() {
    let csv = "name,value\nfoo,1\nbar,2\n";
    let engine = make_csv_engine(csv);

    let rows: Vec<Row> = engine
        .read_csv_records::<Row>()
        .collect::<Result<_, _>>()
        .expect("csv rows should parse");

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
fn csv_row_stream_reports_errors_for_non_csv_format() {
    let registry = default_registry();
    let src = Arc::new(InMemorySource::from_string("json", "{not-csv}"));
    let spec = InputSpec::new("json", src)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let engine = IoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], Vec::new());

    let mut iter = engine.read_csv_records::<Row>();
    let err = iter
        .next()
        .expect("one result")
        .expect_err("expected error for non-csv input");

    assert_eq!(err.stage, crate::error::Stage::ResolveInput);
    assert_eq!(err.target, "json");
}
