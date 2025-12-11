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
