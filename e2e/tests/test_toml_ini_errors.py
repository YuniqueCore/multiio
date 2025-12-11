"""TOML/INI-specific error-path e2e tests for pipeline binaries."""

from pathlib import Path

from conftest import run_pipeline


def test_sync_toml_parse_error(tmp_path: Path, multiio_bin: Path) -> None:
    """Sync pipeline should report a clear parse error for invalid TOML input."""
    invalid_toml = "[service\nname = \"api\"\nport = 8080\n"
    input_file = tmp_path / "invalid.toml"
    input_file.write_text(invalid_toml, encoding="utf-8")

    output_file = tmp_path / "out.json"

    pipeline_yaml = f"""\
inputs:
  - id: bad_toml
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

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)

    assert result.returncode != 0, "invalid TOML pipeline unexpectedly succeeded"
    stderr = result.stderr
    assert "I/O encountered" in stderr
    assert "[Parse]" in stderr
    assert "Serde error" in stderr
    assert "bad_toml" in stderr


def test_sync_ini_parse_error(tmp_path: Path, multiio_bin: Path) -> None:
    """Sync pipeline should report a clear parse error for invalid INI input."""
    invalid_ini = "[service\nname = api\nport = 8080\n"
    input_file = tmp_path / "invalid.ini"
    input_file.write_text(invalid_ini, encoding="utf-8")

    output_file = tmp_path / "out.json"

    pipeline_yaml = f"""\
inputs:
  - id: bad_ini
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

    result = run_pipeline(multiio_bin, pipeline_yaml, tmp_path)

    assert result.returncode != 0, "invalid INI pipeline unexpectedly succeeded"
    stderr = result.stderr
    assert "I/O encountered" in stderr
    assert "[Parse]" in stderr
    assert "Serde error" in stderr
    assert "bad_ini" in stderr


def test_async_toml_parse_error(tmp_path: Path, multiio_async_bin: Path) -> None:
    """Async pipeline should also report a parse error for invalid TOML input."""
    invalid_toml = "[service\nname = \"api\"\nport = 8080\n"
    input_file = tmp_path / "invalid_async.toml"
    input_file.write_text(invalid_toml, encoding="utf-8")

    output_file = tmp_path / "out_async.json"

    pipeline_yaml = f"""\
inputs:
  - id: bad_toml_async
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

    result = run_pipeline(multiio_async_bin, pipeline_yaml, tmp_path)

    assert result.returncode != 0, "invalid TOML async pipeline unexpectedly succeeded"
    stderr = result.stderr
    assert "I/O encountered" in stderr
    assert "[Parse]" in stderr
    assert "Serde error" in stderr
    assert "bad_toml_async" in stderr
