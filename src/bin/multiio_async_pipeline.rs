#![cfg(all(feature = "json", feature = "async"))]

use std::env;
use std::fs::File;
use std::io::BufReader;

use multiio::config::PipelineConfig;
use multiio::{MultiioAsyncBuilder, default_async_registry};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let config_path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: multiio-async-pipeline <pipeline-config.yaml>");
            std::process::exit(1);
        }
    };

    let file = File::open(&config_path)?;
    let reader = BufReader::new(file);
    let config: PipelineConfig = serde_yaml::from_reader(reader)?;

    let registry = default_async_registry();
    let builder = MultiioAsyncBuilder::from_pipeline_config(config, registry)?;
    let engine = builder.build()?;

    // For e2e purposes we operate on serde_json::Value so that any supported
    // format can be used as input or output without having to define a
    // concrete Rust struct in the CLI.
    let mut values: Vec<serde_json::Value> = engine.read_all().await?;

    // Unwrap single-element arrays to avoid double-nesting when the input
    // is already an array (common case for JSON/YAML/CSV inputs).
    if values.len() == 1
        && let serde_json::Value::Array(inner) = &values[0]
    {
        values = inner.clone();
    }

    if values.len() == 1 {
        engine.write_one_value(&values[0]).await?;
    } else {
        engine.write_all(&values).await?;
    }

    Ok(())
}

fn main() {
    if let Err(e) = run_blocking() {
        eprintln!("multiio-async-pipeline error: {e}");
        std::process::exit(1);
    }
}

fn run_blocking() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(run())
}
