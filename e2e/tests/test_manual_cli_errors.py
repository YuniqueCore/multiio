"""Boundary/error behavior e2e tests for multiio_manual CLI."""

import subprocess
from pathlib import Path


def test_manual_cli_missing_arguments_shows_usage(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """Running multiio_manual with no arguments should fail and show usage."""
    result = subprocess.run(
        [str(multiio_manual_bin)],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0, "multiio_manual without args should fail"
    stderr = result.stderr
    assert "multiio_manual error" in stderr
    assert "missing arguments" in stderr
    assert "Usage:" in stderr


def test_manual_cli_multi_in_without_inputs_errors(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """multiio_manual --multi-in without any input paths should fail with a clear error."""
    result = subprocess.run(
        [str(multiio_manual_bin), "--multi-in", "out.json"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0, "multiio_manual --multi-in without inputs should fail"
    stderr = result.stderr
    assert "multiio_manual error" in stderr
    assert "--multi-in requires at least one input" in stderr
    assert "Usage:" in stderr


def test_manual_cli_too_many_args_in_one_to_one_mode(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """multiio_manual <in> <out> extra should fail with too many arguments error."""
    result = subprocess.run(
        [str(multiio_manual_bin), "in.json", "out.json", "extra"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0, "multiio_manual with extra args should fail"
    stderr = result.stderr
    assert "multiio_manual error" in stderr
    assert "too many arguments for one-to-one mode" in stderr
    assert "Usage:" in stderr
