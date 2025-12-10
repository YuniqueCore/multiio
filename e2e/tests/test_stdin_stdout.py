"""Test stdin/stdout pipeline."""

import json
import subprocess
from pathlib import Path

from conftest import e2e_dir, multiio_bin as _multiio_bin, project_root


def test_stdin_json_to_stdout_json(tmp_path: Path, multiio_bin: Path) -> None:
    """Test reading from stdin and writing to stdout."""
    scenario = "stdin_stdout"

    pipeline_yaml = tmp_path / "pipeline.yaml"
    pipeline_yaml.write_text(
        """\
inputs:
  - id: stdin_input
    kind: stdin
    format: json
outputs:
  - id: stdout_output
    kind: stdout
    format: json
error_policy: fast_fail
format_order: ["json", "yaml"]
""",
        encoding="utf-8",
    )

    # multiio_pipeline reads into Vec<Value> and writes Vec<Value>
    # so a JSON array input becomes [[...]] output
    stdin_data = json.dumps([{"msg": "hello"}, {"msg": "world"}])

    root = project_root()
    result = subprocess.run(
        [str(multiio_bin), str(pipeline_yaml)],
        cwd=root,
        capture_output=True,
        text=True,
        input=stdin_data,
    )

    assert result.returncode == 0, f"multiio_pipeline failed: {result.stderr}"

    # Parse stdout as JSON - output is wrapped in an outer array by read_all/write_all
    output = json.loads(result.stdout)
    # The input array is treated as a single value, so we get [[...]]
    expected = [[{"msg": "hello"}, {"msg": "world"}]]

    assert output == expected, f"Unexpected output: {output} != {expected}"
