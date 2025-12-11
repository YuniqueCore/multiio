//! Example demonstrating pipeline configuration-driven I/O.
//!
//! Run with: cargo run --example from_pipeline

use multiio::{ErrorPolicy, MultiioBuilder, PipelineConfig, default_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    name: String,
    value: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load pipeline configuration from a YAML string. The structure matches
    // `PipelineConfig` / `InputConfig` / `OutputConfig` in `multiio::config`.
    let config_str = r#"
inputs:
  - id: config
    kind: file
    path: "examples/data/config.json"
    format: json
outputs:
  - id: out
    kind: stdout
    format: json
error_policy: fast_fail
"#;

    // Parse the pipeline configuration
    let pipeline: PipelineConfig = serde_yaml::from_str(config_str)?;

    println!("Pipeline configuration:");
    println!("  Inputs: {:?}", pipeline.inputs);
    println!("  Outputs: {:?}", pipeline.outputs);
    println!("  Error policy: {:?}", pipeline.error_policy);

    // Build engine from pipeline config
    let registry = default_registry();
    let engine = MultiioBuilder::from_pipeline_config(pipeline, registry)?
        .with_mode(ErrorPolicy::FastFail)
        .build()?;

    // Read all inputs
    let records: Vec<Record> = engine.read_all()?;

    println!("\nRead {} record(s):", records.len());
    for record in &records {
        println!("  - {}: {}", record.name, record.value);
    }

    // Write to outputs
    engine.write_all(&records)?;

    Ok(())
}
