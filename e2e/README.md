## E2E testing

This directory contains the Python/pytest-based end-to-end (e2e) test harness
for the `multiio` project. Tests exercise the compiled Rust CLI binaries
(`multiio_pipeline`, `multiio_async_pipeline`, `multiio_manual`, and
`multiio_records_demo`) against real files on disk.

The harness is intentionally simple and uses `pytest` plus a few small helper
functions in `tests/conftest.py`.

## Requirements

- Python >= 3.13 (see `pyproject.toml`)
- A Rust toolchain with `cargo`
- `pytest` (installed either via `uv` or `pip`)

Using `uv` (recommended):

```bash
cd e2e
uv run pytest -v --tb=short
```

`uv` will create and manage a virtual environment based on `pyproject.toml` and
run `pytest` inside it.

Without `uv`, you can create a venv and install pytest manually:

```bash
cd e2e
python -m venv .venv
source .venv/bin/activate         # bash / zsh
# source .venv/bin/activate.fish  # fish
pip install pytest
pytest -v --tb=short
```

## Directory layout

- `data/input/<scenario>/`
  - Input files for a given scenario (JSON, YAML, CSV, TOML, INI, plaintext,
    etc.).
- `data/output/<scenario>/`
  - Output files produced by the tests. These are created/overwritten on each
    test run.
- `data/output/baseline/<scenario>/`
  - Golden/baseline outputs. Tests compare actual outputs against these files.
- `tests/`
  - `conftest.py` and individual `test_*.py` files that drive the Rust binaries,
    manage temporary pipeline configs, and perform comparisons.

## How the harness works

- `tests/conftest.py` provides shared helpers and fixtures:
  - `multiio_bin` / `multiio_async_bin` / `multiio_manual_bin` /
    `multiio_records_demo_bin` build the corresponding Rust binaries once per
    test session using `cargo build` and return their paths.
  - `run_pipeline` writes a temporary YAML pipeline config to `tmp_path` and
    invokes `multiio_pipeline` or `multiio_async_pipeline` with that file.
  - `run_pipeline_and_compare`:
    - Cleans any existing outputs under `data/output/<scenario>/`.
    - Runs the pipeline binary with the given YAML template.
    - Compares each generated file in `data/output/<scenario>/` against the
      corresponding file under `data/output/baseline/<scenario>/` using either
      JSON or text comparison.
  - `compare_json_files` loads both JSON files, normalizes object key order, and
    asserts equality.
  - `compare_text_files` compares raw text content.

- Individual `test_*.py` files define scenarios such as:
  - Simple JSON/TOML/INI/CSV/YAML roundtrips.
  - Pipeline topologies (1->N, N->1, N->N) mixing multiple formats.
  - Error-path tests for unknown formats, missing files, and invalid syntax.
  - Manual CLI conversions via `multiio_manual`.
  - Records/streaming demos via `multiio_records_demo` (JSONL/NDJSON, CSV, mixed
    JSONL/CSV/YAML inputs).

## Dynamically generated inputs

Some tests intentionally exercise invalid or multi-document inputs without
committing invalid files to the repository:

- TOML/INI error-path tests create invalid documents under `tmp_path` at runtime
  to satisfy pre-commit format checkers while still validating error handling.
- Multi-document YAML inputs for streaming/records demos are also generated in
  temporary files for the same reason.

This keeps the repository-friendly (all committed YAML/TOML files are valid)
while ensuring e2e coverage of failure modes and multi-document streams.

## Running specific tests

You can run individual files or tests using standard `pytest` selection:

```bash
cd e2e
uv run pytest -v --tb=short tests/test_toml.py
uv run pytest -v --tb=short tests/test_records_demo.py::test_records_demo_auto_mixed
```

All tests assume the Rust project root is two levels above this directory and
that `cargo build` can succeed with the required features enabled.
