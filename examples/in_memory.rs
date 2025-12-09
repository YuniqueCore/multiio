//! Example using in-memory I/O for testing.
//!
//! Run with: cargo run --example in_memory

use std::sync::Arc;

use multiio::{
    ErrorPolicy, FormatKind, InMemorySink, InMemorySource, InputSpec, IoEngine, OutputSpec,
    default_registry,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Data {
    id: u32,
    message: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = default_registry();

    // Create in-memory input with JSON data
    let json_data = r#"{"id": 42, "message": "Hello, multiio!"}"#;
    let input_source = Arc::new(InMemorySource::from_string("test-input", json_data));

    // Create in-memory output sink
    let output_sink = Arc::new(InMemorySink::new("test-output"));

    // Build input/output specs manually
    let input_spec = InputSpec::new("test-input", input_source).with_format(FormatKind::Json);

    let output_spec =
        OutputSpec::new("test-output", output_sink.clone()).with_format(FormatKind::Json);

    // Create engine directly
    let engine = IoEngine::new(
        registry,
        ErrorPolicy::FastFail,
        vec![input_spec],
        vec![output_spec],
    );

    // Read data
    let data: Vec<Data> = engine.read_all()?;
    println!("Read data: {:?}", data);

    // Write data back
    engine.write_all(&data)?;

    // Check output
    let output_contents = output_sink.contents_string();
    println!("Output:\n{}", output_contents);

    Ok(())
}
