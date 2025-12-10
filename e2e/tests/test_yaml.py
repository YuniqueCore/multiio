"""Test YAML input/output."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_yaml_to_json(tmp_path: Path, multiio_bin: Path) -> None:
    """Test reading multi-document YAML and writing to JSON."""
    e2e = e2e_dir()
    scenario = "yaml_roundtrip"

    input_file = e2e / "data" / "input" / scenario / "input.yaml"
    output_file = e2e / "data" / "output" / scenario / "output.json"

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: yaml
outputs:
  - id: out
    kind: file
    path: {output_file}
    format: json
error_policy: fast_fail
format_order: ["yaml", "json"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )
