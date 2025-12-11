"""Non-pipeline CLI e2e tests for multiio_manual."""

import json
import subprocess
from pathlib import Path

from conftest import e2e_dir, project_root, compare_json_files


def test_manual_one_in_one_out_json(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """1 input JSON array -> 1 JSON array output via multiio_manual."""
    e2e = e2e_dir()
    root = project_root()
    scenario = "manual_one_in_one_out"

    input_file = e2e / "data" / "input" / scenario / "input.json"
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

    assert result.returncode == 0, f"multiio_manual failed: {result.stderr}\nStdout: {result.stdout}"
    assert output_file.exists(), "manual CLI did not create output file"

    baseline_file = e2e / "data" / "output" / "baseline" / scenario / "output.json"
    compare_json_files(output_file, baseline_file)


def test_manual_stdin_to_stdout_json(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """stdin JSON array -> stdout JSON array via multiio_manual (- -)."""
    root = project_root()

    # Use an array to exercise the "unwrap" logic consistently with pipeline
    # behavior (arrays remain arrays).
    stdin_values = [
        {"name": "alice", "age": 30},
        {"name": "bob", "age": 25},
    ]
    stdin_data = json.dumps(stdin_values)

    result = subprocess.run(
        [str(multiio_manual_bin), "-", "-"],
        cwd=root,
        capture_output=True,
        text=True,
        input=stdin_data,
    )

    assert result.returncode == 0, f"multiio_manual stdin->stdout failed: {result.stderr}\nStdout: {result.stdout}"

    output = json.loads(result.stdout)
    assert output == stdin_values


def test_manual_multi_in_stdin_and_files(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """stdin JSON object + 2 JSON files -> 1 JSON array via multiio_manual --multi-in.

    This exercises a mixed-input scenario where one source is stdin ("-") and the
    others are regular files, with the combined records written to a JSON file.
    """
    e2e = e2e_dir()
    root = project_root()
    scenario = "manual_multi_in_stdin_and_files"

    input1 = e2e / "data" / "input" / scenario / "input1.json"
    input2 = e2e / "data" / "input" / scenario / "input2.json"
    output_dir = e2e / "data" / "output" / scenario
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / "output.json"

    if output_file.exists():
        output_file.unlink()

    # Stdin provides an additional JSON object which should become the first
    # element of the output array.
    stdin_value = {"name": "stdin", "age": 99}
    stdin_data = json.dumps(stdin_value)

    result = subprocess.run(
        [
            str(multiio_manual_bin),
            "--multi-in",
            str(output_file),
            "-",  # stdin
            str(input1),
            str(input2),
        ],
        cwd=root,
        capture_output=True,
        text=True,
        input=stdin_data,
    )

    assert (
        result.returncode == 0
    ), f"multiio_manual --multi-in (stdin + files) failed: {result.stderr}\nStdout: {result.stdout}"
    assert output_file.exists(), "manual CLI did not create multi-in (stdin+files) output file"

    baseline_file = e2e / "data" / "output" / "baseline" / scenario / "output.json"
    compare_json_files(output_file, baseline_file)


def test_manual_multi_in_one_out_json(tmp_path: Path, multiio_manual_bin: Path) -> None:
    """2 JSON object inputs -> 1 JSON array output via multiio_manual --multi-in."""
    e2e = e2e_dir()
    root = project_root()
    scenario = "manual_multi_in_one_out"

    input1 = e2e / "data" / "input" / scenario / "input1.json"
    input2 = e2e / "data" / "input" / scenario / "input2.json"
    output_dir = e2e / "data" / "output" / scenario
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / "output.json"

    if output_file.exists():
        output_file.unlink()

    result = subprocess.run(
        [
            str(multiio_manual_bin),
            "--multi-in",
            str(output_file),
            str(input1),
            str(input2),
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_manual --multi-in failed: {result.stderr}\nStdout: {result.stdout}"
    assert output_file.exists(), "manual CLI did not create multi-in output file"

    baseline_file = e2e / "data" / "output" / "baseline" / scenario / "output.json"
    compare_json_files(output_file, baseline_file)
