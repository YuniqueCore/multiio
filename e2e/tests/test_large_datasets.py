"""Large / more complex dataset e2e tests for sync and async pipelines."""

import json
from pathlib import Path

from conftest import compare_json_files, run_pipeline


def _write_large_json_array(path: Path, n: int = 1000) -> None:
    """Helper to write a reasonably large JSON array for stability testing."""
    records = [
        {
            "id": i,
            "name": f"user-{i}",
            "scores": [i, i + 1, i + 2],
            "meta": {"active": i % 2 == 0, "groups": ["g1", "g2"] if i % 10 == 0 else []},
        }
        for i in range(n)
    ]
    path.write_text(json.dumps(records), encoding="utf-8")


def test_large_json_roundtrip_sync(tmp_path: Path, multiio_bin: Path) -> None:
    """Sync pipeline should handle a larger JSON array roundtrip stably."""
    input_file = tmp_path / "large_input.json"
    output_file = tmp_path / "large_output.json"

    _write_large_json_array(input_file, n=1000)

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: json
outputs:
  - id: out
    kind: file
    path: {output_file}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "plaintext"]
"""

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)
    assert result.returncode == 0, f"large sync pipeline failed: {result.stderr}\nStdout: {result.stdout}"

    # Use shared JSON comparator to ignore key order differences
    compare_json_files(output_file, input_file)


def test_large_json_roundtrip_async(tmp_path: Path, multiio_async_bin: Path) -> None:
    """Async pipeline should also handle a larger JSON array roundtrip stably."""
    input_file = tmp_path / "large_input_async.json"
    output_file = tmp_path / "large_output_async.json"

    _write_large_json_array(input_file, n=1000)

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: json
outputs:
  - id: out
    kind: file
    path: {output_file}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "plaintext"]
"""

    result = run_pipeline(multiio_async_bin, pipeline_yaml, tmp_path)
    assert result.returncode == 0, f"large async pipeline failed: {result.stderr}\nStdout: {result.stdout}"

    compare_json_files(output_file, input_file)
