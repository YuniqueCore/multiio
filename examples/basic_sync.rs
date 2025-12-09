//! Basic synchronous example demonstrating multiio usage.
//!
//! Run with: cargo run --example basic_sync

use multiio::{ErrorPolicy, MultiioBuilder, default_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    name: String,
    value: i32,
    enabled: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a default format registry with JSON, YAML, etc.
    let registry = default_registry();

    // Build an I/O engine with inputs and outputs
    let engine = MultiioBuilder::new(registry)
        .add_input("examples/data/config.json")
        .add_output("-") // stdout
        .with_mode(ErrorPolicy::FastFail)
        .build()?;

    // Read all inputs (each input becomes one Config)
    let configs: Vec<Config> = engine.read_all()?;

    println!("Read {} config(s):", configs.len());
    for config in &configs {
        println!(
            "  - {}: {} (enabled: {})",
            config.name, config.value, config.enabled
        );
    }

    // Write all configs to outputs
    engine.write_all(&configs)?;

    Ok(())
}
