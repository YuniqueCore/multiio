"""Shared pytest fixtures and helpers for e2e tests."""

import json
import subprocess
from pathlib import Path
from typing import Any

import pytest


def project_root() -> Path:
    """Return the multiio project root directory."""
    return Path(__file__).resolve().parents[2]


def e2e_dir() -> Path:
    """Return the e2e directory."""
    return Path(__file__).resolve().parents[1]


@pytest.fixture(scope="session")
def multiio_bin() -> Path:
    """Ensure the multiio_pipeline binary is built once per test session."""
    root = project_root()
    result = subprocess.run(
        ["cargo", "build", "--quiet", "--bin", "multiio_pipeline"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"cargo build failed: {result.stderr}"

    bin_path = root / "target" / "debug" / "multiio_pipeline"
    assert bin_path.exists(), f"built binary not found at {bin_path}"

    return bin_path


def run_pipeline(
    multiio_bin: Path,
    pipeline_yaml_content: str,
    tmp_path: Path,
    stdin_data: str | None = None,
) -> subprocess.CompletedProcess:
    """Run multiio_pipeline with the given YAML config and optional stdin.

    Args:
        multiio_bin: Path to the built multiio_pipeline binary
        pipeline_yaml_content: YAML pipeline configuration content
        tmp_path: Temporary directory for the pipeline config file
        stdin_data: Optional string to feed to stdin

    Returns:
        CompletedProcess instance with returncode, stdout, stderr
    """
    pipeline_yaml = tmp_path / "pipeline.yaml"
    pipeline_yaml.write_text(pipeline_yaml_content, encoding="utf-8")

    root = project_root()
    result = subprocess.run(
        [str(multiio_bin), str(pipeline_yaml)],
        cwd=root,
        capture_output=True,
        text=True,
        input=stdin_data,
    )

    return result


def compare_json_files(actual_path: Path, baseline_path: Path) -> None:
    """Compare two JSON files for equality."""
    actual = json.loads(actual_path.read_text(encoding="utf-8"))
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    assert actual == baseline, f"JSON mismatch: {actual} != {baseline}"


def compare_text_files(actual_path: Path, baseline_path: Path) -> None:
    """Compare two text files for equality."""
    actual = actual_path.read_text(encoding="utf-8")
    baseline = baseline_path.read_text(encoding="utf-8")
    assert actual == baseline, f"Text mismatch:\nActual:\n{actual}\nBaseline:\n{baseline}"


def run_pipeline_and_compare(
    scenario: str,
    pipeline_template: str,
    multiio_bin: Path,
    tmp_path: Path,
    output_files: dict[str, str],  # {output_id: extension}
    stdin_data: str | None = None,
    comparator: str = "json",  # "json" or "text"
) -> None:
    """Run a pipeline scenario and compare outputs to baselines.

    Args:
        scenario: Scenario name (used for data/input/<scenario> and data/output/baseline/<scenario>)
        pipeline_template: YAML template for the pipeline (should use {input_file}, {output_file}, etc.)
        multiio_bin: Path to the built binary
        tmp_path: Temporary directory
        output_files: Dict mapping output IDs to file extensions
        stdin_data: Optional stdin data
        comparator: "json" or "text" comparison
    """
    e2e = e2e_dir()

    # Prepare output directory
    output_dir = e2e / "data" / "output" / scenario
    output_dir.mkdir(parents=True, exist_ok=True)

    # Clean up existing output files
    for ext in output_files.values():
        output_file = output_dir / f"output.{ext}"
        if output_file.exists():
            output_file.unlink()

    # Run pipeline
    result = run_pipeline(multiio_bin, pipeline_template, tmp_path, stdin_data)
    assert result.returncode == 0, f"multiio_pipeline failed: {result.stderr}\nStdout: {result.stdout}"

    # Compare each output to baseline
    baseline_dir = e2e / "data" / "output" / "baseline" / scenario
    for output_id, ext in output_files.items():
        output_file = output_dir / f"output.{ext}"
        baseline_file = baseline_dir / f"output.{ext}"

        assert output_file.exists(), f"output file {output_file} was not created"
        assert baseline_file.exists(), f"baseline file {baseline_file} does not exist"

        if comparator == "json":
            compare_json_files(output_file, baseline_file)
        else:
            compare_text_files(output_file, baseline_file)
