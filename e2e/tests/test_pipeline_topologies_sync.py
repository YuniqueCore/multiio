"""Sync pipeline topology e2e tests (1->N, N->1, N->N)."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_sync_one_in_multi_out_json_yaml_csv(tmp_path: Path, multiio_bin: Path) -> None:
    """1 input -> 3 outputs (json, yaml, csv) using pipeline config."""
    e2e = e2e_dir()
    scenario = "json_multi_outputs_sync"

    input_file = e2e / "data" / "input" / scenario / "input.json"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: json
outputs:
  - id: out_json
    kind: file
    path: {output_dir / 'output.json'}
    format: json
  - id: out_yaml
    kind: file
    path: {output_dir / 'output.yaml'}
    format: yaml
  - id: out_csv
    kind: file
    path: {output_dir / 'output.csv'}
    format: csv
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={
            "out_json": "json",
            "out_yaml": "yaml",
            "out_csv": "csv",
        },
        comparator="json",
    )


def test_sync_multi_in_single_out_json(tmp_path: Path, multiio_bin: Path) -> None:
    """2 JSON inputs -> 1 JSON output using pipeline config."""
    e2e = e2e_dir()
    scenario = "multi_in_single_out_sync"

    input1 = e2e / "data" / "input" / scenario / "input1.json"
    input2 = e2e / "data" / "input" / scenario / "input2.json"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in1
    kind: file
    path: {input1}
    format: json
  - id: in2
    kind: file
    path: {input2}
    format: json
outputs:
  - id: out
    kind: file
    path: {output_dir / 'output.json'}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )


def test_sync_multi_in_multi_out_json(tmp_path: Path, multiio_bin: Path) -> None:
    """2 JSON inputs -> 2 JSON outputs using pipeline config."""
    e2e = e2e_dir()
    scenario = "multi_in_multi_out_sync"

    input1 = e2e / "data" / "input" / scenario / "input1.json"
    input2 = e2e / "data" / "input" / scenario / "input2.json"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in1
    kind: file
    path: {input1}
    format: json
  - id: in2
    kind: file
    path: {input2}
    format: json
outputs:
  - id: out1
    kind: file
    path: {output_dir / 'output1.json'}
    format: json
  - id: out2
    kind: file
    path: {output_dir / 'output2.json'}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={
            "out1": "json",
            "out2": "json",
        },
        comparator="json",
    )
