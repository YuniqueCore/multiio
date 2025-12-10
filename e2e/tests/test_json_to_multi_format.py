"""Test JSON input to multiple output formats."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_json_to_csv_yaml_markdown(tmp_path: Path, multiio_bin: Path) -> None:
    """Test reading JSON and writing to CSV, YAML, and Markdown."""
    e2e = e2e_dir()
    scenario = "json_to_multi_format"

    input_file = e2e / "data" / "input" / scenario / "input.json"
    output_csv = e2e / "data" / "output" / scenario / "output.csv"
    output_yaml = e2e / "data" / "output" / scenario / "output.yaml"
    output_md = e2e / "data" / "output" / scenario / "output.md"

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: json
outputs:
  - id: csv_out
    kind: file
    path: {output_csv}
    format: csv
  - id: yaml_out
    kind: file
    path: {output_yaml}
    format: yaml
  - id: md_out
    kind: file
    path: {output_md}
    format: markdown
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "markdown", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"csv_out": "csv", "yaml_out": "yaml", "md_out": "md"},
        comparator="text",
    )
