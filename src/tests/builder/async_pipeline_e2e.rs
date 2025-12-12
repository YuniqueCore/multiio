#![cfg(feature = "json")]

use std::path::PathBuf;

use crate::config::PipelineConfig;
use crate::{ErrorPolicy, MultiioAsyncBuilder, default_async_registry};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct ConfigData {
    name: String,
    value: i32,
}

async fn write_json_object(path: &PathBuf, data: &ConfigData) {
    let json = serde_json::to_string_pretty(data).expect("serialize test data");
    tokio::fs::write(path, json)
        .await
        .expect("write test input file");
}

#[tokio::test]
async fn async_pipeline_e2e_file_to_file_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("input.json");
    let out_path = dir.path().join("output.json");

    let record = ConfigData {
        name: "a".into(),
        value: 1,
    };
    write_json_object(&in_path, &record).await;

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
        out_path.to_string_lossy(),
    );

    let pipeline: PipelineConfig = serde_yaml::from_str(&yaml).expect("parse pipeline yaml");

    let registry = default_async_registry();
    let builder = MultiioAsyncBuilder::from_pipeline_config(pipeline, registry)
        .expect("from_pipeline_config should succeed");

    let engine = builder
        .with_mode(ErrorPolicy::FastFail)
        .build()
        .expect("build async engine");

    let vals: Vec<ConfigData> = engine.read_all().await.expect("read_all");
    assert_eq!(vals.len(), 1);
    assert_eq!(vals[0], record);

    engine
        .write_all(&vals)
        .await
        .expect("write_all should succeed");

    let out_bytes = tokio::fs::read(&out_path).await.expect("read output file");
    let decoded: Vec<ConfigData> = serde_json::from_slice(&out_bytes).expect("decode output json");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0], record);
}
