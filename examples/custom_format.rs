//! Example demonstrating custom format registration.
//!
//! Run with: cargo run --example custom_format

use multiio::{
    CustomFormat, ErrorPolicy, FormatError, FormatKind, InMemorySink, InMemorySource, InputSpec,
    IoEngine, OutputSpec, default_registry,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Config {
    name: String,
    value: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a registry with default formats
    let mut registry = default_registry();

    // Register a custom "uppercase-json" format that serializes JSON in uppercase
    let uppercase_json = CustomFormat::new("uppercase-json", &["ujson"])
        .with_deserialize(|bytes| {
            // Parse as regular JSON (case doesn't matter for parsing)
            serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_serialize(|value| {
            // Serialize to JSON, then uppercase it
            let json_str =
                serde_json::to_string_pretty(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
            Ok(json_str.to_uppercase().into_bytes())
        });

    registry.register_custom(uppercase_json);

    // Create in-memory input with JSON data
    let json_data = r#"{"name": "test", "value": 42}"#;
    let input_source = Arc::new(InMemorySource::from_string("input", json_data));

    // Create in-memory output sink
    let output_sink = Arc::new(InMemorySink::new("output"));

    // Build specs using the custom format
    let input_spec = InputSpec::new("input", input_source).with_format(FormatKind::Json);

    let output_spec = OutputSpec::new("output", output_sink.clone())
        .with_format(FormatKind::custom("uppercase-json"));

    // Create engine
    let engine = IoEngine::new(
        registry,
        ErrorPolicy::FastFail,
        vec![input_spec],
        vec![output_spec],
    );

    // Read data
    let configs: Vec<Config> = engine.read_all()?;
    println!("Read: {:?}", configs);

    // Write using custom format
    engine.write_all(&configs)?;

    // Check output (should be uppercase JSON)
    let output = output_sink.contents_string();
    println!("\nOutput (uppercase JSON):\n{}", output);

    Ok(())
}
