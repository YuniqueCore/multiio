#![cfg(feature = "json")]

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
fn pipeline_e2e_multi_input_multi_output_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path1 = dir.path().join("input1.json");
    let in_path2 = dir.path().join("input2.json");
    let out_path1 = dir.path().join("output1.json");
    let out_path2 = dir.path().join("output2.json");

    let record1 = ConfigData {
        name: "a".into(),
        value: 1,
    };
    let record2 = ConfigData {
        name: "b".into(),
        value: 2,
    };
    write_json_object(&in_path1, &record1);
    write_json_object(&in_path2, &record2);

    let yaml = format!(
        r#"
inputs:
  - id: in1
    kind: file
    path: {}
    format: json
  - id: in2
    kind: file
    path: {}
    format: json
outputs:
  - id: out1
    kind: file
    path: {}
    format: json
  - id: out2
    kind: file
    path: {}
    format: json
error_policy: accumulate
format_order: ["json", "yaml", "plaintext"]
"#,
        in_path1.to_string_lossy(),
        in_path2.to_string_lossy(),
        out_path1.to_string_lossy(),
        out_path2.to_string_lossy(),
    );

    let pipeline: PipelineConfig = serde_yaml::from_str(&yaml).expect("parse pipeline yaml");

    let registry = default_registry();
    let builder = MultiioBuilder::from_pipeline_config(pipeline, registry)
        .expect("from_pipeline_config should succeed");

    let engine = builder
        .with_mode(ErrorPolicy::Accumulate)
        .build()
        .expect("build engine");

    let vals: Vec<ConfigData> = engine.read_all().expect("read_all");
    assert_eq!(vals.len(), 2);
    assert_eq!(vals[0], record1);
    assert_eq!(vals[1], record2);

    engine.write_all(&vals).expect("write_all");

    for out in [&out_path1, &out_path2] {
        let out_bytes = fs::read(out).expect("read output file");
        let decoded: Vec<ConfigData> =
            serde_json::from_slice(&out_bytes).expect("decode output json");
        assert_eq!(decoded, vals);
    }
}

#[test]
fn pipeline_unknown_custom_format_accumulates_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let good_path = dir.path().join("good.json");
    let bad_path = dir.path().join("bad.json");

    let record = ConfigData {
        name: "good".into(),
        value: 1,
    };
    write_json_object(&good_path, &record);
    write_json_object(&bad_path, &record);

    let yaml = format!(
        r#"
inputs:
  - id: good
    kind: file
    path: {}
    format: json
  - id: bad
    kind: file
    path: {}
    format: custom:missing-format
outputs:
  - id: out
    kind: stdout
error_policy: accumulate
format_order: ["json"]
"#,
        good_path.to_string_lossy(),
        bad_path.to_string_lossy(),
    );

    let pipeline: PipelineConfig = serde_yaml::from_str(&yaml).expect("parse pipeline yaml");

    let registry = default_registry();
    let builder = MultiioBuilder::from_pipeline_config(pipeline, registry)
        .expect("from_pipeline_config should succeed");

    let engine = builder
        .with_mode(ErrorPolicy::Accumulate)
        .build()
        .expect("build engine");

    let result: Result<Vec<ConfigData>, AggregateError> = engine.read_all();
    let agg = result.expect_err("expected aggregate error due to unknown custom format");

    assert_eq!(agg.errors.len(), 1);
    let e = &agg.errors[0];
    assert_eq!(e.stage, Stage::Parse);
    assert_eq!(e.target, "bad");
    let msg = e.error.to_string();
    assert!(msg.contains("Unknown format"));
    assert!(msg.contains("missing-format"));
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

    fs::write(&out_path, b"OLD").expect("write initial output");

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

    let contents = fs::read(&out_path).expect("read output");
    let s = String::from_utf8_lossy(&contents);
    assert!(s.starts_with("OLD"));
}
