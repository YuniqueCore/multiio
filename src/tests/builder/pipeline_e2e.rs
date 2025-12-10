//! End-to-end tests for MultiioBuilder::from_pipeline_config.

use std::fs;
use std::path::PathBuf;

use crate::config::PipelineConfig;
use crate::error::{AggregateError, Stage};
use crate::{ErrorPolicy, MultiioBuilder, default_registry};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct ConfigData {
    name: String,
    value: i32,
}

fn write_json_object(path: &PathBuf, data: &ConfigData) {
    let json = serde_json::to_string_pretty(data).expect("serialize test data");
    fs::write(path, json).expect("write test input file");
}

#[test]
fn pipeline_e2e_file_to_file_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("input.json");
    let out_path = dir.path().join("output.json");

    let record = ConfigData {
        name: "a".into(),
        value: 1,
    };
    write_json_object(&in_path, &record);

    let yaml = format!(
        r#"
inputs:
  - id: in
    kind: file
    path: {}
    format: json
outputs:
  - id: out
    kind: file
    path: {}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "plaintext"]
"#,
        in_path.to_string_lossy(),
        out_path.to_string_lossy()
    );

    let pipeline: PipelineConfig = serde_yaml::from_str(&yaml).expect("parse pipeline yaml");

    let registry = default_registry();
    let builder = MultiioBuilder::from_pipeline_config(pipeline, registry)
        .expect("from_pipeline_config should succeed");

    // Override error policy explicitly to be safe
    let engine = builder
        .with_mode(ErrorPolicy::FastFail)
        .build()
        .expect("build engine");

    let vals: Vec<ConfigData> = engine.read_all().expect("read_all");
    assert_eq!(vals.len(), 1);
    assert_eq!(vals[0], record);

    engine.write_all(&vals).expect("write_all");

    let out_bytes = fs::read(&out_path).expect("read output file");
    let decoded: Vec<ConfigData> = serde_json::from_slice(&out_bytes).expect("decode output json");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0], record);
}

#[test]
fn pipeline_unknown_input_kind_produces_resolve_error() {
    let yaml = r#"
inputs:
  - id: net1
    kind: http
    url: "https://example.com/data.json"
outputs:
  - id: out
    kind: stdout
"#;

    let pipeline: PipelineConfig = serde_yaml::from_str(yaml).expect("parse pipeline yaml");

    let registry = default_registry();
    let result = MultiioBuilder::from_pipeline_config(pipeline, registry);

    let err: AggregateError = match result {
        Err(e) => e,
        Ok(_) => panic!("expected failure due to unknown input kind"),
    };
    assert_eq!(err.errors.len(), 1);
    let e = &err.errors[0];
    assert_eq!(e.stage, Stage::ResolveInput);
    assert_eq!(e.target, "net1");
}

#[test]
fn pipeline_missing_file_path_for_file_input() {
    let yaml = r#"
inputs:
  - id: in
    kind: file
outputs:
  - id: out
    kind: stdout
"#;

    let pipeline: PipelineConfig = serde_yaml::from_str(yaml).expect("parse pipeline yaml");

    let registry = default_registry();
    let result = MultiioBuilder::from_pipeline_config(pipeline, registry);

    let err: AggregateError = match result {
        Err(e) => e,
        Ok(_) => panic!("expected failure due to missing file path"),
    };
    assert_eq!(err.errors.len(), 1);
    let e = &err.errors[0];
    assert_eq!(e.stage, Stage::ResolveInput);
    assert_eq!(e.target, "in");
}

#[test]
fn pipeline_file_exists_policy_from_string() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_path = dir.path().join("out.json");

    let yaml = format!(
        r#"
inputs:
  - id: in
    kind: stdin
outputs:
  - id: out
    kind: file
    path: {}
    file_exists_policy: append
"#,
        out_path.to_string_lossy()
    );

    let pipeline: PipelineConfig = serde_yaml::from_str(&yaml).expect("parse pipeline yaml");

    let registry = default_registry();
    let builder = MultiioBuilder::from_pipeline_config(pipeline, registry)
        .expect("from_pipeline_config should succeed");

    // Pre-write some data to the output file
    fs::write(&out_path, b"OLD").expect("write initial output");

    // Build engine; we won't call read_all() (stdin), only write_all()
    let engine = builder
        .with_mode(ErrorPolicy::FastFail)
        .build()
        .expect("build engine");

    #[derive(Debug, serde::Serialize)]
    struct Dummy {
        x: i32,
    }

    let vals = vec![Dummy { x: 1 }];
    engine.write_all(&vals).expect("write_all");

    // With append policy, new JSON bytes should be appended after existing content
    let contents = fs::read(&out_path).expect("read output");
    let s = String::from_utf8_lossy(&contents);
    assert!(s.starts_with("OLD"));
}
