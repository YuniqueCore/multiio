"""TOML/INI manual CLI e2e tests for multiio_manual."""

import subprocess
from pathlib import Path

from conftest import e2e_dir, project_root, compare_json_files


def test_manual_toml_one_in_one_out(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """1 TOML input -> 1 JSON output via multiio_manual."""
    e2e = e2e_dir()
    root = project_root()
    scenario = "manual_toml_one_in_one_out"

    input_file = e2e / "data" / "input" / scenario / "input.toml"
    output_dir = e2e / "data" / "output" / scenario
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / "output.json"

    if output_file.exists():
        output_file.unlink()

    result = subprocess.run(
        [str(multiio_manual_bin), str(input_file), str(output_file)],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_manual TOML failed: {result.stderr}\nStdout: {result.stdout}"
    assert output_file.exists(), "manual CLI TOML case did not create output file"

    baseline_file = e2e / "data" / "output" / "baseline" / scenario / "output.json"
    compare_json_files(output_file, baseline_file)


def test_manual_ini_one_in_one_out(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """1 INI input -> 1 JSON output via multiio_manual."""
    e2e = e2e_dir()
    root = project_root()
    scenario = "manual_ini_one_in_one_out"

    input_file = e2e / "data" / "input" / scenario / "input.ini"
    output_dir = e2e / "data" / "output" / scenario
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / "output.json"

    if output_file.exists():
        output_file.unlink()

    result = subprocess.run(
        [str(multiio_manual_bin), str(input_file), str(output_file)],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_manual INI failed: {result.stderr}\nStdout: {result.stdout}"
    assert output_file.exists(), "manual CLI INI case did not create output file"

    baseline_file = e2e / "data" / "output" / "baseline" / scenario / "output.json"
    compare_json_files(output_file, baseline_file)
