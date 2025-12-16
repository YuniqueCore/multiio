"""E2E tests for the sarge-based CLI demo (multiio_sarge)."""

import json
import subprocess
from pathlib import Path

from conftest import project_root


def _baseline_json(scenario: str, filename: str = "output.json") -> object:
    root = project_root()
    path = root / "e2e" / "data" / "output" / "baseline" / scenario / filename
    return json.loads(path.read_text(encoding="utf-8"))


def test_sarge_inline_json_to_stdout(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_token = '{"a":1}'

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", input_token, "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""

    output = json.loads(result.stdout)
    assert output == [{"a": 1}]


def test_sarge_stdin_alias_to_stdout(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    stdin_data = '{"a":1}'

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", "stdin", "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
        input=stdin_data,
    )

    assert result.returncode == 0, f"multiio_sarge stdin->stdout failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""

    output = json.loads(result.stdout)
    assert output == [{"a": 1}]


def test_sarge_writes_to_stderr(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_token = '{"a":1}'

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", input_token, "--output", "stderr"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge ->stderr failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stdout == ""

    output = json.loads(result.stderr)
    assert output == [{"a": 1}]


def test_sarge_forced_path_creates_file(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    output_path = tmp_path / "stderr"
    input_token = '{"a":1}'

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", input_token, "--output", f"@{output_path}"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge forced-path failed: {result.stderr}\nStdout: {result.stdout}"
    assert output_path.exists(), "expected forced-path output file to be created"

    output = json.loads(output_path.read_text(encoding="utf-8"))
    assert output == [{"a": 1}]


def test_sarge_repeatable_outputs_stdout_and_stderr(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_token = '{"a":1}'

    result = subprocess.run(
        [
            str(multiio_sarge_bin),
            "--input",
            input_token,
            "--output",
            "stdout",
            "--output",
            "stderr",
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge repeatable outputs failed: {result.stderr}\nStdout: {result.stdout}"

    out = json.loads(result.stdout)
    err = json.loads(result.stderr)
    assert out == [{"a": 1}]
    assert err == [{"a": 1}]


def test_sarge_repeatable_inputs_with_inline_json_commas(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()

    # First token includes a comma, which used to break naive comma-splitting.
    in1 = '{"a":1,"b":2}'
    in2 = '{"c":3}'

    result = subprocess.run(
        [
            str(multiio_sarge_bin),
            "--input",
            in1,
            "--input",
            in2,
            "--output",
            "stdout",
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge repeatable inputs failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""

    output = json.loads(result.stdout)
    assert output == [{"a": 1, "b": 2}, {"c": 3}]


def test_sarge_reads_yaml_file_and_outputs_json(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_file = root / "e2e" / "data" / "input" / "yaml_roundtrip" / "input.yaml"
    expected = _baseline_json("yaml_roundtrip")

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", str(input_file), "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge yaml->json failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""
    assert json.loads(result.stdout) == [expected]


def test_sarge_reads_toml_file_and_outputs_json(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_file = root / "e2e" / "data" / "input" / "toml_roundtrip" / "input.toml"
    expected = _baseline_json("toml_roundtrip")

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", str(input_file), "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge toml->json failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""
    assert json.loads(result.stdout) == [expected]


def test_sarge_reads_ini_file_and_outputs_json(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_file = root / "e2e" / "data" / "input" / "ini_roundtrip" / "input.ini"
    expected = _baseline_json("ini_roundtrip")

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", str(input_file), "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge ini->json failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""
    assert json.loads(result.stdout) == [expected]


def test_sarge_reads_csv_file_and_outputs_json(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_file = root / "e2e" / "data" / "input" / "csv_roundtrip" / "input.csv"
    expected = _baseline_json("csv_roundtrip")

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", str(input_file), "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge csv->json failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""
    assert json.loads(result.stdout) == [expected]


def test_sarge_reads_plaintext_file_and_outputs_json(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_file = root / "e2e" / "data" / "input" / "plaintext_lines" / "input.txt"
    expected = _baseline_json("plaintext_lines")

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", str(input_file), "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert (
        result.returncode == 0
    ), f"multiio_sarge plaintext->json failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""
    assert json.loads(result.stdout) == [expected]


def test_sarge_reads_xml_file_and_outputs_json(tmp_path: Path, multiio_sarge_bin: Path) -> None:
    root = project_root()
    input_file = root / "e2e" / "data" / "input" / "xml_roundtrip" / "input.xml"
    expected = _baseline_json("xml_roundtrip")

    result = subprocess.run(
        [str(multiio_sarge_bin), "--input", str(input_file), "--output", "stdout"],
        cwd=root,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, f"multiio_sarge xml->json failed: {result.stderr}\nStdout: {result.stdout}"
    assert result.stderr == ""
    assert json.loads(result.stdout) == [expected]
