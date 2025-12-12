"""Exhaustive feature-matrix compilation test.

This test enumerates all subsets of independent crate features and runs
`cargo check --tests` for each subset with `--no-default-features`.

The goal is to ensure optional dependencies and cfg-gating remain correct for
every possible feature combination.
"""

import itertools
import os
import subprocess
from pathlib import Path

import pytest


def project_root() -> Path:
    return Path(__file__).resolve().parents[2]


BASE_FEATURES = [
    "json",
    "yaml",
    "toml",
    "ini",
    "csv",
    "xml",
    "plaintext",
    "async",
    "miette",
    "custom",
]


def run_check(features: tuple[str, ...], *, no_default: bool) -> None:
    root = project_root()
    manifest = root / "Cargo.toml"
    target_dir = root / "target" / "feature-matrix-e2e"

    cmd = [
        "cargo",
        "check",
        "--tests",
        "--quiet",
        "--manifest-path",
        str(manifest),
    ]
    if no_default:
        cmd.append("--no-default-features")
    if features:
        cmd.extend(["--features", ",".join(features)])

    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    env["RUSTFLAGS"] = "-D warnings"

    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        env=env,
    )

    assert result.returncode == 0, (
        f"cargo check failed for no_default={no_default}, features={features}\n"
        f"stdout:\n{result.stdout}\n\nstderr:\n{result.stderr}"
    )


def test_exhaustive_feature_matrix_compiles() -> None:
    if os.environ.get("MULTIIO_EXHAUSTIVE_FEATURE_MATRIX") != "1":
        pytest.skip("set MULTIIO_EXHAUSTIVE_FEATURE_MATRIX=1 to enable exhaustive feature checks")

    # Default feature set should compile.
    run_check((), no_default=False)

    # Exhaustively check all subsets without default features.
    for r in range(len(BASE_FEATURES) + 1):
        for combo in itertools.combinations(BASE_FEATURES, r):
            run_check(combo, no_default=True)

    # Umbrella feature should compile on its own.
    run_check(("full",), no_default=True)
