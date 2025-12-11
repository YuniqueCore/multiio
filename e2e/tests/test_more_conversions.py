"""Additional format conversion e2e tests (YAML->TOML, TOML->INI)."""

from pathlib import Path

from conftest import e2e_dir, run_pipeline_and_compare


def test_yaml_to_toml_sync(tmp_path: Path, multiio_bin: Path) -> None:
    """YAML input -> TOML output using sync pipeline."""
    e2e = e2e_dir()
    scenario = "yaml_to_toml_sync"

    input_file = e2e / "data" / "input" / scenario / "input.yaml"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: yaml
outputs:
  - id: out
    kind: file
    path: {output_dir / 'output.toml'}
    format: toml
error_policy: fast_fail
format_order: ["yaml", "toml", "json", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "toml"},
        comparator="text",
    )


def test_toml_to_ini_sync(tmp_path: Path, multiio_bin: Path) -> None:
    """TOML input -> INI output using sync pipeline."""
    e2e = e2e_dir()
    scenario = "toml_to_ini_sync"

    input_file = e2e / "data" / "input" / scenario / "input.toml"
    output_dir = e2e / "data" / "output" / scenario

    pipeline_yaml = f"""\
inputs:
  - id: in
    kind: file
    path: {input_file}
    format: toml
outputs:
  - id: out
    kind: file
    path: {output_dir / 'output.ini'}
    format: ini
error_policy: fast_fail
format_order: ["toml", "ini", "json", "plaintext"]
"""

    run_pipeline_and_compare(
        scenario=scenario,
        pipeline_template=pipeline_yaml,
        multiio_bin=multiio_bin,
        tmp_path=tmp_path,
        output_files={"out": "ini"},
        comparator="text",
    )
