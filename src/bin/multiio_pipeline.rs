#![cfg(all(feature = "json", feature = "yaml"))]

use std::env;
use std::fs::File;
use std::io::BufReader;

use multiio::MultiioBuilder;
use multiio::config::PipelineConfig;
use multiio::format::default_registry;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let config_path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: multiio-pipeline <pipeline-config.yaml>");
            std::process::exit(1);
        }
    };

    let file = File::open(&config_path)?;
    let reader = BufReader::new(file);
    let config: PipelineConfig = serde_yaml::from_reader(reader)?;

    let registry = default_registry();
    let builder = MultiioBuilder::from_pipeline_config(config, registry)?;
    let engine = builder.build()?;

    // For e2e purposes we operate on serde_json::Value so that any supported
    // format can be used as input or output without having to define a
    // concrete Rust struct in the CLI.
    let values: Vec<serde_json::Value> = engine.read_all()?;
    engine.write_all(&values)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        // AggregateError already implements Display, so we can show a concise
        // summary here.
        eprintln!("multiio-pipeline error: {e}");
        // Use non-zero exit code so that e2e tests can detect failure.
        std::process::exit(1);
    }
}
