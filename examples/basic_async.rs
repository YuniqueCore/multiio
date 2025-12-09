//! Basic asynchronous example demonstrating multiio usage.
//!
//! Run with: cargo run --example basic_async --features async

#[cfg(feature = "async")]
mod async_example {
    use multiio::{ErrorPolicy, MultiioAsyncBuilder, default_async_registry};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Config {
        name: String,
        value: i32,
        enabled: bool,
    }

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        // Create a default async format registry
        let registry = default_async_registry();

        // Build an async I/O engine
        let engine = MultiioAsyncBuilder::new(registry)
            .add_input("examples/data/config.json")
            .add_output("-") // stdout
            .with_mode(ErrorPolicy::FastFail)
            .build()?;

        // Read all inputs asynchronously
        let configs: Vec<Config> = engine.read_all().await?;

        println!("Read {} config(s):", configs.len());
        for config in &configs {
            println!(
                "  - {}: {} (enabled: {})",
                config.name, config.value, config.enabled
            );
        }

        // Write all configs to outputs
        engine.write_all(&configs).await?;

        Ok(())
    }
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_example::run().await
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature. Run with:");
    eprintln!("  cargo run --example basic_async --features async");
}
