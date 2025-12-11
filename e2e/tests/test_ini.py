"""Test INI input/output."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_ini_to_json(tmp_path: Path, multiio_bin: Path) -> None:
    """Test reading INI and writing to JSON."""
    e2e = e2e_dir()
    scenario = "ini_roundtrip"

    input_file = e2e / "data" / "input" / scenario / "input.ini"
    output_file = e2e / "data" / "output" / scenario / "output.json"

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: ini
outputs:
  - id: out
    kind: file
    path: {output_file}
    format: json
error_policy: fast_fail
format_order: ["ini", "json", "yaml", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )
