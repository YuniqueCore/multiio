use std::env;
use std::error::Error;

use multiio::{ErrorPolicy, MultiioBuilder};

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: multiio_records_demo <mode> <input>");
    eprintln!("  <mode>: json | csv | auto");
    eprintln!("  <input>: file path or '-' for stdin");
    std::process::exit(1);
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let mode = match args.next() {
        Some(m) => m,
        None => return Err("missing <mode> argument (json|csv|auto)".into()),
    };

    let first_input = match args.next() {
        Some(p) => p,
        None => return Err("missing <input> argument".into()),
    };

    let mut inputs = vec![first_input];
    inputs.extend(args);

    let mut builder = MultiioBuilder::default().with_mode(ErrorPolicy::FastFail);
    for input in inputs {
        builder = builder.add_input(input);
    }

    let engine = builder.build()?;

    match mode.as_str() {
        "json" => {
            for res in engine.read_json_records::<serde_json::Value>() {
                let value = res.map_err(|e| format!("record error: {e}"))?;
                println!("{}", serde_json::to_string(&value)?);
            }
        }
        "csv" => {
            for res in engine.read_csv_records::<serde_json::Value>() {
                let value = res.map_err(|e| format!("record error: {e}"))?;
                println!("{}", serde_json::to_string(&value)?);
            }
        }
        "auto" => {
            for res in engine.read_records::<serde_json::Value>() {
                let value = res.map_err(|e| format!("record error: {e}"))?;
                println!("{}", serde_json::to_string(&value)?);
            }
        }
        _ => {
            return Err(format!("unknown mode: {} (expected json|csv|auto)", mode).into());
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("multiio_records_demo error: {e}");
        print_usage_and_exit();
    }
}
