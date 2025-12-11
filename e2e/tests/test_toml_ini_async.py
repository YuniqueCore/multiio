"""Async pipeline e2e tests for TOML and INI formats."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_toml_to_json_async(tmp_path: Path, multiio_async_bin: Path) -> None:
    """Async pipeline: TOML input -> JSON output."""
    e2e = e2e_dir()
    scenario = "toml_roundtrip"

    input_file = e2e / "data" / "input" / scenario / "input.toml"
    output_file = e2e / "data" / "output" / scenario / "output.json"

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: toml
outputs:
  - id: out
    kind: file
    path: {output_file}
    format: json
error_policy: fast_fail
format_order: ["toml", "json", "yaml", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_async_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )


def test_ini_to_json_async(tmp_path: Path, multiio_async_bin: Path) -> None:
    """Async pipeline: INI input -> JSON output."""
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
        multiio_bin=multiio_async_bin,
        tmp_path=tmp_path,
        output_files={"out": "json"},
        comparator="json",
    )
