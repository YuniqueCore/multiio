"""Error-path e2e tests for pipeline binaries (sync + async)."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline


def test_sync_pipeline_unknown_custom_input_format(tmp_path: Path, multiio_bin: Path) -> None:
    """Pipeline with custom:missing-format input should fail with clear error."""
    e2e = e2e_dir()
    scenario = "simple_json_roundtrip"

    input_file = e2e / "data" / "input" / scenario / "input.json"
    output_json = tmp_path / "out.json"

    pipeline_yaml = f"""\
inputs:
  - id: good
    kind: file
    path: {input_file}
    format: json
  - id: bad
    kind: file
    path: {input_file}
    format: custom:missing-format
outputs:
  - id: out
    kind: file
    path: {output_json}
    format: json
error_policy: accumulate
format_order: ["json"]
"""

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)

    assert result.returncode != 0, "pipeline with unknown custom format unexpectedly succeeded"
    stderr = result.stderr
    assert "Unknown format" in stderr
    assert "missing-format" in stderr
    assert "bad" in stderr


def test_sync_pipeline_missing_input_file(tmp_path: Path, multiio_bin: Path) -> None:
    """Pipeline should fail cleanly when input file does not exist."""
    missing = tmp_path / "missing_input.json"
    output_json = tmp_path / "output.json"

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {missing}
    format: json
outputs:
  - id: out
    kind: file
    path: {output_json}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "plaintext"]
"""

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)

    assert result.returncode != 0, "pipeline with missing input unexpectedly succeeded"
    stderr = result.stderr
    assert "I/O encountered" in stderr
    assert "[Open]" in stderr
    # Engine reports the input id ("in"), not the full file path, as the target
    assert " in:" in stderr


def test_sync_pipeline_append_file_exists_policy_appends(tmp_path: Path, multiio_bin: Path) -> None:
    """file_exists_policy: append should preserve existing content when writing."""
    e2e = e2e_dir()
    scenario = "simple_json_roundtrip"

    input_file = e2e / "data" / "input" / scenario / "input.json"
    output_json = tmp_path / "append_output.json"

    # Pre-create file with some content to verify append behavior
    output_json.write_text("OLD\n", encoding="utf-8")

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: json
outputs:
  - id: out
    kind: file
    path: {output_json}
    format: json
    file_exists_policy: append
error_policy: fast_fail
format_order: ["json", "yaml", "plaintext"]
"""

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)
    assert result.returncode == 0, f"pipeline with append policy failed: {result.stderr}\nStdout: {result.stdout}"

    contents = output_json.read_text(encoding="utf-8")
    assert contents.startswith("OLD"), "append policy did not preserve existing prefix"

