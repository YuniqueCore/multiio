use std::path::PathBuf;

use crate::config::PipelineConfig;
use crate::format::{self, FormatKind};
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

#[tokio::test]
async fn async_pipeline_e2e_mixed_formats_json_to_csv_md_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("input.json");
    let csv_out = dir.path().join("out.csv");
    let md_out = dir.path().join("out.md");
    let yaml_out = dir.path().join("out.yaml");

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
  - id: csv
    kind: file
    path: {}
    format: csv
  - id: md
    kind: file
    path: {}
    format: markdown
  - id: yaml
    kind: file
    path: {}
    format: yaml
error_policy: fast_fail
format_order: ["json", "csv", "markdown", "yaml"]
"#,
        in_path.to_string_lossy(),
        csv_out.to_string_lossy(),
        md_out.to_string_lossy(),
        yaml_out.to_string_lossy(),
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

    // CSV output
    let csv_bytes = tokio::fs::read(&csv_out).await.expect("read csv output");
    let csv_vals: Vec<ConfigData> =
        format::deserialize(FormatKind::Csv, &csv_bytes).expect("deserialize csv");
    assert_eq!(csv_vals, vals);

    // Markdown output
    let md_bytes = tokio::fs::read(&md_out)
        .await
        .expect("read markdown output");
    let md_vals: Vec<ConfigData> =
        format::deserialize(FormatKind::Markdown, &md_bytes).expect("deserialize markdown");
    assert_eq!(md_vals, vals);

    // YAML output
    let yaml_bytes = tokio::fs::read(&yaml_out).await.expect("read yaml output");
    let yaml_vals: Vec<ConfigData> =
        format::deserialize(FormatKind::Yaml, &yaml_bytes).expect("deserialize yaml");
    assert_eq!(yaml_vals, vals);
}
