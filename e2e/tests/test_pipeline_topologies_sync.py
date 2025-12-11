"""Sync pipeline topology e2e tests (1->N, N->1, N->N)."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline, run_pipeline_and_compare, compare_json_files


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

    # First, compare JSON output structurally
    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out_json": "json"},
        comparator="json",
    )

    # Then, compare YAML and CSV outputs as plain text
    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={
            "out_yaml": "yaml",
            "out_csv": "csv",
        },
        comparator="text",
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

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)
    assert result.returncode == 0, f"multiio_pipeline failed: {result.stderr}\nStdout: {result.stdout}"

    baseline_dir = e2e / "data" / "output" / "baseline" / scenario

    output1 = output_dir / "output1.json"
    output2 = output_dir / "output2.json"
    baseline1 = baseline_dir / "output1.json"
    baseline2 = baseline_dir / "output2.json"

    assert output1.exists() and baseline1.exists()
    assert output2.exists() and baseline2.exists()

    compare_json_files(output1, baseline1)
    compare_json_files(output2, baseline2)


def test_sync_toml_one_in_multi_out_json_yaml(tmp_path: Path, multiio_bin: Path) -> None:
    """1 TOML input -> 2 outputs (json, yaml) using pipeline config."""
    e2e = e2e_dir()
    scenario = "toml_multi_outputs_sync"

    input_file = e2e / "data" / "input" / scenario / "input.toml"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: toml
outputs:
  - id: out_json
    kind: file
    path: {output_dir / 'output.json'}
    format: json
  - id: out_yaml
    kind: file
    path: {output_dir / 'output.yaml'}
    format: yaml
error_policy: fast_fail
format_order: ["toml", "json", "yaml", "plaintext"]
"""

    # First compare JSON structurally
    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out_json": "json"},
        comparator="json",
    )

    # Then compare YAML as plain text
    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out_yaml": "yaml"},
        comparator="text",
    )


def test_sync_json_to_toml_single_out(tmp_path: Path, multiio_bin: Path) -> None:
    """JSON input -> TOML output (format conversion)."""
    e2e = e2e_dir()
    scenario = "json_to_toml_sync"

    input_file = e2e / "data" / "input" / scenario / "input.json"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: json
outputs:
  - id: out
    kind: file
    path: {output_dir / 'output.toml'}
    format: toml
error_policy: fast_fail
format_order: ["json", "toml", "yaml", "plaintext"]
"""

    # Compare TOML output as plain text
    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "toml"},
        comparator="text",
    )


def test_sync_mixed_multi_in_single_out_json(tmp_path: Path, multiio_bin: Path) -> None:
    """Mixed JSON/TOML/INI inputs -> single JSON output."""
    e2e = e2e_dir()
    scenario = "mixed_multi_in_single_out_sync"

    input_json = e2e / "data" / "input" / scenario / "input_json.json"
    input_toml = e2e / "data" / "input" / scenario / "input.toml"
    input_ini = e2e / "data" / "input" / scenario / "input.ini"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: json_in
    kind: file
    path: {input_json}
    format: json
  - id: toml_in
    kind: file
    path: {input_toml}
    format: toml
  - id: ini_in
    kind: file
    path: {input_ini}
    format: ini
outputs:
  - id: out
    kind: file
    path: {output_dir / 'output.json'}
    format: json
error_policy: fast_fail
format_order: ["json", "toml", "ini", "yaml", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )
