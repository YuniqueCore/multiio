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

| Feature     | Description               | Default |
| ----------- | ------------------------- | ------- |
| `json`      | JSON format support       | ✓       |
| `yaml`      | YAML format support       | ✓       |
| `csv`       | CSV format support        | ✓       |
| `plaintext` | Plaintext format support  | ✓       |
| `toml`      | TOML format support       | ✓       |
| `ini`       | INI/".ini" config support | ✓       |
| `xml`       | XML format support        |         |
| `markdown`  | Markdown format support   |         |
| `async`     | Async I/O with Tokio      |         |
| `miette`    | Pretty error reporting    |         |
| `full`      | All features              |         |

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

## CLI binaries

multiio ships with a few small CLI binaries that exercise the core APIs and
serve as real-world examples:

- `multiio_pipeline`
  - Sync pipeline runner driven by a YAML config file (see "Pipeline
    configuration" above).
  - Used in the e2e tests under `e2e/tests/test_*.py` to verify 1->N, N->1, and
    N->N topologies across JSON/YAML/CSV/TOML/INI.
- `multiio_async_pipeline`
  - Async variant of the pipeline runner. Takes the same YAML config format but
    runs on top of `MultiioAsyncBuilder`.
- `multiio_manual`
  - Minimal non-pipeline CLI for quick format conversions:
    - `multiio_manual <input> <output>`
    - `multiio_manual --multi-in <output> <input1> <input2> [...]`
  - Formats are inferred from file extensions and feature flags
    (JSON/YAML/CSV/TOML/INI/plaintext).
- `multiio_records_demo`
  - Demo CLI for the streaming/records APIs (`read_json_records`,
    `read_csv_records`, `read_records`).
  - Prints one JSON document per record (NDJSON) to stdout:
    - `multiio_records_demo json <input.jsonl>`
    - `multiio_records_demo csv <input.csv>`
    - `multiio_records_demo auto <input1> <input2> [...]` (mixed
      JSONL/CSV/YAML).

All binaries are optional and can be enabled/disabled via Cargo features and bin
targets in `Cargo.toml`.

### Format matrix

The following table summarizes which formats and engine modes each CLI uses by
default (assuming the corresponding Cargo features are enabled):

| CLI                      | Sync | Async | JSON | YAML | CSV | TOML | INI | Plaintext |
| ------------------------ | ---- | ----- | ---- | ---- | --- | ---- | --- | --------- |
| `multiio_pipeline`       | ✓    |       | ✓    | ✓    | ✓   | ✓    | ✓   | ✓         |
| `multiio_async_pipeline` |      | ✓     | ✓    | ✓    | ✓   | ✓    | ✓   | ✓         |
| `multiio_manual`         | ✓    |       | ✓    | ✓    | ✓   | ✓    | ✓   | ✓         |
| `multiio_records_demo`   | ✓    |       | ✓    | ✓    | ✓   |      |     |           |

Notes:

- Actual availability depends on enabling the corresponding Cargo features (see
  the Features table above).
- `multiio_records_demo` focuses on the streaming/records APIs and is intended
  primarily for JSONL/NDJSON logs, CSV rows and YAML documents.

## E2E tests

The repository contains a Python/pytest-based end-to-end (e2e) test harness in
the `e2e/` directory. It drives the CLI binaries, compares outputs against
golden files, and exercises real filesystem I/O.

Directory layout:

- `e2e/data/input/<scenario>/` – input files for a given scenario
- `e2e/data/output/<scenario>/` – outputs generated during tests
- `e2e/data/output/baseline/<scenario>/` – golden/baseline outputs
- `e2e/tests/` – pytest test cases and helpers

Some representative scenarios:

- Pipeline topologies
  - 1->N, N->1 and N->N flows using `multiio_pipeline` (sync) and
    `multiio_async_pipeline` (async).
  - Mixes JSON, YAML, CSV, plaintext, TOML and INI formats.
- Format conversions
  - JSON<->YAML/CSV/TOML/INI and cross-format conversions such as YAML->TOML or
    TOML->INI.
- Error paths
  - Invalid inputs, unknown/disabled formats, and conflicting outputs are
    covered to ensure good error reporting.
- Manual CLI
  - `multiio_manual` is tested for 1->1 and multi-in->1 conversions, including
    TOML/INI inputs.
- Records demo
  - `multiio_records_demo` is tested with JSONL, CSV and mixed JSONL/CSV/YAML
    inputs to validate the streaming/records APIs in a CLI setting.

Refer to the files under `e2e/tests/` for concrete test cases and example
pipeline configurations.

## Usage examples

### Example: Streaming NDJSON logs

Given a log file `logs.jsonl` (one JSON object per line), you can stream and
filter records using the records demo CLI plus standard tools such as `jq`:

```bash
multiio_records_demo json logs.jsonl \
  | jq 'select(.level == "error")'
```

This keeps the CLI focused on reliable I/O and format handling while leaving
domain-specific querying to dedicated tools.

### Example: Aggregating JSON/TOML/INI configs

You can aggregate configuration files from multiple formats into a single JSON
file using the pipeline runner. For example, a `configs.yaml` pipeline:

```yaml
inputs:
  - id: json
    kind: file
    path: config.json
    format: json
  - id: toml
    kind: file
    path: config.toml
    format: toml
  - id: ini
    kind: file
    path: config.ini
    format: ini
outputs:
  - id: merged
    kind: file
    path: merged.json
    format: json
error_policy: fast_fail
```

Running:

```bash
multiio_pipeline configs.yaml
```

will read all three config files and write them as a JSON sequence (or array),
ready for further processing or inspection.

## License

MIT OR Apache-2.0
