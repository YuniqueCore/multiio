"""Test simple JSON file roundtrip."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_simple_json_roundtrip(tmp_path: Path, multiio_bin: Path) -> None:
    """Test reading JSON file and writing to JSON file."""
    e2e = e2e_dir()
    scenario = "simple_json_roundtrip"

    input_file = e2e / "data" / "input" / scenario / "input.json"
    output_file = e2e / "data" / "output" / scenario / "output.json"

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
format_order: ["json", "yaml", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )
