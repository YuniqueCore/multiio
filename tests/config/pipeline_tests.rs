//! Tests for PipelineConfig parsing and builder helpers.

use multiio::config::PipelineConfig;

#[test]
fn parse_minimal_pipeline_config() {
    let yaml = r#"
inputs:
  - id: in
    kind: file
    path: input.json
outputs:
  - id: out
    kind: file
    path: output.json
"#;

    let cfg: PipelineConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.inputs.len(), 1);
    assert_eq!(cfg.outputs.len(), 1);
    assert_eq!(cfg.inputs[0].id, "in");
    assert_eq!(cfg.outputs[0].id, "out");
}
