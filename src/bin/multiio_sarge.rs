use std::error::Error;

use multiio::cli::{InputArgs, OutputArgs};
use multiio::format::default_registry;
use multiio::{ErrorPolicy, MultiioBuilder};
use sarge::prelude::*;

fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  multiio_sarge --input <token> [--input <token> ...] --output <token> [--output <token> ...]"
    );
    eprintln!();
    eprintln!("Input tokens:");
    eprintln!("  - | stdin          Read from stdin");
    eprintln!("  =<content>         Inline content (in-memory input)");
    eprintln!("  @<path>            Force treating value as a file path");
    eprintln!();
    eprintln!("Output tokens:");
    eprintln!("  - | stdout         Write to stdout");
    eprintln!("  stderr             Write to stderr");
    eprintln!("  @<path>            Force treating value as a file path");
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut reader = ArgumentReader::new();

    let input_ref = reader.add::<InputArgs>(tag::both('i', "input"));
    let output_ref = reader.add::<OutputArgs>(tag::both('o', "output"));

    let args = reader.parse()?;

    let input = match input_ref.get(&args) {
        Some(Ok(v)) => v,
        Some(Err(_)) => unreachable!("InputArgs parsing is infallible"),
        None => InputArgs::default(),
    };

    let output = match output_ref.get(&args) {
        Some(Ok(v)) => v,
        Some(Err(_)) => unreachable!("OutputArgs parsing is infallible"),
        None => OutputArgs::default(),
    };

    if input.is_empty() || output.is_empty() {
        return Err("missing --input/--output".into());
    }

    let registry = default_registry();
    let engine = MultiioBuilder::new(registry)
        .with_mode(ErrorPolicy::FastFail)
        .with_input_args(&input)
        .with_output_args(&output)
        .build()?;

    let values: Vec<serde_json::Value> = engine.read_all()?;
    engine.write_all(&values)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("multiio_sarge error: {e}");
        print_usage();
        std::process::exit(1);
    }
}
