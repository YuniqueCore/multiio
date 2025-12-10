# multiio

A unified I/O orchestration library for CLI and server applications in Rust.

## Overview

multiio provides a clean abstraction for handling multiple inputs and outputs
with automatic format detection and cross-format read/write (for example, JSON
inputs fanned out to CSV/Markdown/YAML outputs). It supports both synchronous
and asynchronous I/O patterns.

### Key Features

- **Multi-input/Multi-output**: Read from and write to multiple sources
  simultaneously
- **Format Abstraction**: Built-in support for JSON, YAML, CSV, XML, Markdown,
  and plaintext
- **Custom Formats**: Register your own formats via `CustomFormat` and
  `FormatRegistry`, including custom file extensions
- **Sync and Async**: Both synchronous and asynchronous I/O support
- **Error Handling**: Configurable error policies (FastFail or Accumulate)
- **Pipeline Configuration**: Define I/O workflows via YAML/JSON config files
- **In-Memory I/O**: Testing-friendly in-memory sources and sinks

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
multiio = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Example

```rust
use multiio::{default_registry, ErrorPolicy, MultiioBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    name: String,
    value: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = default_registry();

    let engine = MultiioBuilder::new(registry)
        .add_input("config.json")
        .add_output("-")  // stdout
        .with_mode(ErrorPolicy::FastFail)
        .build()?;

    let configs: Vec<Config> = engine.read_all()?;
    engine.write_all(&configs)?;

    Ok(())
}
```

### Async Example

Enable the `async` feature:

```toml
[dependencies]
multiio = { version = "0.1", features = ["async"] }
```

```rust
use multiio::{default_async_registry, ErrorPolicy, MultiioAsyncBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    name: String,
    value: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = default_async_registry();

    let engine = MultiioAsyncBuilder::new(registry)
        .add_input("config.yaml")
        .add_output("output.json")
        .with_mode(ErrorPolicy::FastFail)
        .build()?;

    let configs: Vec<Config> = engine.read_all().await?;
    engine.write_all(&configs).await?;

    Ok(())
}
```

## Features

| Feature     | Description              | Default |
| ----------- | ------------------------ | ------- |
| `json`      | JSON format support      | ✓       |
| `yaml`      | YAML format support      | ✓       |
| `csv`       | CSV format support       | ✓       |
| `plaintext` | Plaintext format support | ✓       |
| `xml`       | XML format support       |         |
| `markdown`  | Markdown format support  |         |
| `async`     | Async I/O with Tokio     |         |
| `miette`    | Pretty error reporting   |         |
| `full`      | All features             |         |

## Custom formats

multiio allows you to register your own formats using `CustomFormat` and
`FormatRegistry`. Custom formats use `serde_json::Value` as an intermediate
representation so they can participate in the normal `read_all`/`write_all`
pipeline alongside the built-in formats.

```rust
use multiio::format::{CustomFormat, FormatError, FormatRegistry};

let mut registry = FormatRegistry::new();

let bracket = CustomFormat::new("bracket", &["brk"])
    .with_deserialize(|bytes| {
        // Very simple example: strip leading/trailing brackets and parse JSON
        let s = String::from_utf8_lossy(bytes);
        let inner = s.trim_start_matches('[').trim_end_matches(']');
        serde_json::from_str(inner).map_err(|e| FormatError::Serde(Box::new(e)))
    })
    .with_serialize(|value| {
        let json = serde_json::to_string(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
        Ok(format!("[{json}]" ).into_bytes())
    });

registry.register_custom(bracket);
```

Custom formats can then be selected via `FormatKind::Custom("bracket")` or by
registering file extensions (such as `".brk"`) and letting the registry infer
the format from the path.

## Pipeline configuration (YAML)

In addition to building engines programmatically, you can drive multiio from a
YAML pipeline configuration. This is useful for CLIs or tools that need
configurable I/O workflows.

```yaml
inputs:
  - id: in
    kind: file
    path: input.json
    format: json
outputs:
  - id: out
    kind: file
    path: output.yaml
    format: yaml
error_policy: fast_fail
format_order: ["json", "yaml", "plaintext"]
```

```rust
use multiio::{default_registry, MultiioBuilder, PipelineConfig};

let yaml = std::fs::read_to_string("pipeline.yaml")?;
let pipeline: PipelineConfig = serde_yaml::from_str(&yaml)?;

let registry = default_registry();
let builder = MultiioBuilder::from_pipeline_config(pipeline, registry)?;
let engine = builder.build()?;

// Use the engine as usual
let values: Vec<serde_json::Value> = engine.read_all()?;
engine.write_all(&values)?;
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    MultiioBuilder                    │
│         (parses args, creates InputSpec/OutputSpec)  │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│                      IoEngine                        │
│              (orchestrates I/O operations)           │
└──────────┬────────────────────────────┬─────────────┘
           │                            │
           ▼                            ▼
┌──────────────────────┐    ┌──────────────────────────┐
│    InputProvider     │    │      OutputTarget        │
│  (StdinInput,        │    │  (StdoutOutput,          │
│   FileInput, etc.)   │    │   FileOutput, etc.)      │
└──────────────────────┘    └──────────────────────────┘
           │                            │
           ▼                            ▼
┌─────────────────────────────────────────────────────┐
│                   FormatRegistry                     │
│     (JSON, YAML, CSV, XML, Markdown, Plaintext)     │
└─────────────────────────────────────────────────────┘
```

## Error Handling

multiio supports two error policies:

- **FastFail**: Stop at the first error encountered
- **Accumulate**: Collect all errors and return them together

```rust
let engine = MultiioBuilder::new(registry)
    .with_mode(ErrorPolicy::Accumulate)  // Collect all errors
    .build()?;
```

With the `miette` feature, errors can be displayed with pretty formatting.

## License

MIT OR Apache-2.0
