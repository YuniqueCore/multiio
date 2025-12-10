"""Async pipeline e2e tests using multiio_async_pipeline binary."""

from pathlib import Path

from conftest import (
    e2e_dir,
    run_pipeline,
    run_pipeline_and_compare,
    compare_json_files,
)


def test_async_simple_json_roundtrip_pipeline(tmp_path: Path, multiio_async_bin: Path) -> None:
    """Async: JSON file -> JSON file using pipeline config."""
    e2e = e2e_dir()
    scenario = "simple_json_roundtrip"

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
    path: {output_dir / 'output.json'}
    format: json
error_policy: fast_fail
format_order: ["json", "yaml", "csv", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_async_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )


def test_async_multi_in_multi_out_json(tmp_path: Path, multiio_async_bin: Path) -> None:
    """Async: 2 JSON inputs -> 2 JSON outputs using same pipeline as sync test."""
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

    # Run async pipeline once
    result = run_pipeline(multiio_async_bin, pipeline_yaml, tmp_path)
    assert result.returncode == 0, f"multiio_async_pipeline failed: {result.stderr}\nStdout: {result.stdout}"

    baseline_dir = e2e / "data" / "output" / "baseline" / scenario

    output1 = output_dir / "output1.json"
    output2 = output_dir / "output2.json"
    baseline1 = baseline_dir / "output1.json"
    baseline2 = baseline_dir / "output2.json"

    assert output1.exists() and baseline1.exists()
    assert output2.exists() and baseline2.exists()

    compare_json_files(output1, baseline1)
    compare_json_files(output2, baseline2)
