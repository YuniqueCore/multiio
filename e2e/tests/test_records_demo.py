"""Records streaming demo CLI e2e tests for multiio_records_demo."""

import json
import subprocess
from pathlib import Path

from conftest import e2e_dir, project_root


def _run_demo(mode: str, input_path: Path, multiio_records_demo_bin: Path) -> list[dict]:
    root = project_root()

    result = subprocess.run(
        [str(multiio_records_demo_bin), mode, str(input_path)],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert (
        result.returncode == 0
    ), f"multiio_records_demo {mode} failed: {result.stderr}\nStdout: {result.stdout}"

    lines = [line for line in result.stdout.splitlines() if line.strip()]
    return [json.loads(line) for line in lines]


def _run_demo_multi(mode: str, input_paths: list[Path], multiio_records_demo_bin: Path) -> list[dict]:
    """Run records demo CLI with multiple inputs and parse JSON lines."""
    root = project_root()

    cmd = [str(multiio_records_demo_bin), mode] + [str(p) for p in input_paths]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert (
        result.returncode == 0
    ), f"multiio_records_demo {mode} failed: {result.stderr}\nStdout: {result.stdout}"

    lines = [line for line in result.stdout.splitlines() if line.strip()]
    return [json.loads(line) for line in lines]


def test_records_demo_csv(tmp_path: Path, multiio_records_demo_bin: Path) -> None:
    """CSV input -> JSON lines via records demo (csv mode)."""
    e2e = e2e_dir()
    input_file = e2e / "data" / "input" / "records_csv_demo" / "input.csv"

    records = _run_demo("csv", input_file, multiio_records_demo_bin)

    # Depending on how CSV rows are deserialized, each record may be a simple
    # string (e.g. first column) or an object. For the demo we only care that
    # the logical "name" field streams in order.
    names: list[str] = []
    for r in records:
        if isinstance(r, str):
            names.append(r)
        elif isinstance(r, dict) and "name" in r:
            names.append(str(r["name"]))
        else:
            raise AssertionError(f"Unexpected CSV record shape: {r!r}")

    assert names == ["alice", "bob"]


def test_records_demo_json(tmp_path: Path, multiio_records_demo_bin: Path) -> None:
    """JSONL input -> JSON lines via records demo (json mode)."""
    e2e = e2e_dir()
    input_file = e2e / "data" / "input" / "records_json_demo" / "input.jsonl"

    records = _run_demo("json", input_file, multiio_records_demo_bin)

    assert records == [
        {"name": "alice", "age": 30},
        {"name": "bob", "age": 25},
    ]


def test_records_demo_auto_mixed(tmp_path: Path, multiio_records_demo_bin: Path) -> None:
    """Mixed JSONL/CSV/YAML inputs -> JSON lines via records demo (auto mode)."""
    e2e = e2e_dir()
    jsonl_file = e2e / "data" / "input" / "records_json_demo" / "input.jsonl"
    csv_file = e2e / "data" / "input" / "records_csv_demo" / "input.csv"

    # Use a multi-document YAML file generated in a temporary directory so that
    # repository-committed YAML stays single-document and compatible with
    # pre-commit's check-yaml hook.
    yaml_file = tmp_path / "records_yaml_demo.yaml"
    yaml_file.write_text(
        "---\n"
        "name: carol\n"
        "age: 40\n"
        "---\n"
        "name: dave\n"
        "age: 35\n"
    )

    records = _run_demo_multi(
        "auto",
        [jsonl_file, csv_file, yaml_file],
        multiio_records_demo_bin,
    )

    # All records should stream in input order: JSONL docs, then CSV rows, then
    # YAML documents. We only assert the logical "name" sequence.
    names: list[str] = []
    for r in records:
        if isinstance(r, str):
            names.append(r)
        elif isinstance(r, dict) and "name" in r:
            names.append(str(r["name"]))
        else:
            raise AssertionError(f"Unexpected record shape in auto mode: {r!r}")

    assert names == ["alice", "bob", "alice", "bob", "carol", "dave"]
