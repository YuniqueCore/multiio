# multiio

A unified I/O orchestration library for CLI and server applications in Rust.

## Overview

multiio provides a clean abstraction for handling multiple inputs and outputs
with automatic format detection and conversion. It supports both synchronous and
asynchronous I/O patterns.

### Key Features

- **Multi-input/Multi-output**: Read from and write to multiple sources
  simultaneously
- **Format Abstraction**: Built-in support for JSON, YAML, CSV, XML, Markdown,
  and plaintext
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = default_async_registry();

    let engine = MultiioAsyncBuilder::new(registry)
        .add_input("config.yaml")
        .add_output("output.json")
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
