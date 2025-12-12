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

    let mut values: Vec<serde_json::Value> = engine.read_all()?;

    if values.len() == 1 {
        if let serde_json::Value::Array(inner) = &values[0] {
            values = inner.clone();
        }
    }

    if values.len() == 1 {
        engine.write_one_value(&values[0])?;
    } else {
        engine.write_all(&values)?;
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("multiio-pipeline error: {e}");
        std::process::exit(1);
    }
}
