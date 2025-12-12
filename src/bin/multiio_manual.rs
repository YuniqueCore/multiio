use std::env;
use std::error::Error;

use multiio::{ErrorPolicy, MultiioBuilder};

fn print_usage_and_exit() -> ! {
    eprintln!("Usage:");
    eprintln!("  multiio_manual <input> <output>");
    eprintln!("  multiio_manual --multi-in <output> <input1> <input2> [...]");
    std::process::exit(1);
}

fn run_one_to_one(input: String, output: String) -> Result<(), Box<dyn Error>> {
    let builder = MultiioBuilder::default()
        .add_input(input)
        .add_output(output)
        .with_mode(ErrorPolicy::FastFail);

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

fn run_many_to_one(inputs: Vec<String>, output: String) -> Result<(), Box<dyn Error>> {
    let mut builder = MultiioBuilder::default().with_mode(ErrorPolicy::FastFail);

    for input in inputs {
        builder = builder.add_input(input);
    }

    builder = builder.add_output(output);

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

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let first = match args.next() {
        Some(arg) => arg,
        None => return Err("missing arguments".into()),
    };

    if first == "--multi-in" {
        let output = match args.next() {
            Some(o) => o,
            None => return Err("--multi-in requires an output path".into()),
        };

        let inputs: Vec<String> = args.collect();
        if inputs.is_empty() {
            return Err("--multi-in requires at least one input".into());
        }

        run_many_to_one(inputs, output)
    } else {
        let input = first;
        let output = match args.next() {
            Some(o) => o,
            None => return Err("missing output argument".into()),
        };

        if args.next().is_some() {
            return Err("too many arguments for one-to-one mode".into());
        }

        run_one_to_one(input, output)
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("multiio_manual error: {e}");
        print_usage_and_exit();
    }
}
