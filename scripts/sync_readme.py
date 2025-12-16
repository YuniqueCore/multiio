#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path


def run_cargo_metadata(manifest_path: Path) -> dict:
    cargo = os.environ.get("CARGO", "cargo")

    cmd = [
        cargo,
        "metadata",
        "--no-deps",
        "--format-version",
        "1",
        "--manifest-path",
        str(manifest_path),
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise RuntimeError(
            "cargo metadata failed.\n"
            f"cmd: {' '.join(cmd)}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}\n"
        )
    return json.loads(result.stdout)


def select_root_package(metadata: dict, manifest_path: Path) -> dict:
    manifest_path = manifest_path.resolve()
    packages = metadata.get("packages", [])
    if not isinstance(packages, list):
        raise ValueError("cargo metadata returned invalid packages list.")
    for pkg in packages:
        if not isinstance(pkg, dict):
            continue
        mp = pkg.get("manifest_path")
        if not isinstance(mp, str):
            continue
        if Path(mp).resolve() == manifest_path:
            return pkg
    raise ValueError(f"Could not find package for manifest path: {manifest_path}")


def parse_feature_keys_from_manifest(manifest_text: str) -> list[str]:
    in_features = False
    keys: list[str] = []
    for raw_line in manifest_text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            in_features = line == "[features]"
            continue
        if not in_features:
            continue
        # Stop when leaving the [features] section.
        if line.startswith("["):
            break
        # Strip trailing comments (good enough for our simple keys parsing).
        line = line.split("#", 1)[0].strip()
        match = re.match(r"^([A-Za-z0-9_-]+)\s*=", line)
        if match:
            keys.append(match.group(1))
    return keys


def generate_features_table(feature_keys: list[str], features_map: dict) -> list[str]:
    default_features = features_map.get("default", [])
    if default_features is None:
        default_features = []
    if not isinstance(default_features, list):
        raise ValueError("cargo metadata features.default must be an array.")
    default_set = {f for f in default_features if isinstance(f, str)}

    feature_names = [name for name in feature_keys if name != "default"]
    preferred_order = [
        "plaintext",
        "json",
        "yaml",
        "toml",
        "ini",
        "xml",
        "csv",
        "custom",
        "async",
        "miette",
        "sarge",
        "full",
    ]
    ordered: list[str] = []
    for name in preferred_order:
        if name in feature_names:
            ordered.append(name)
            feature_names.remove(name)
    ordered.extend(sorted(feature_names))

    descriptions: dict[str, str] = {
        "plaintext": "Plaintext format support",
        "json": "JSON format support",
        "yaml": "YAML format support",
        "toml": "TOML format support",
        "ini": "INI/\".ini\" config support",
        "xml": "XML format support",
        "csv": "CSV format support",
        "custom": "Custom formats via registry",
        "async": "Async I/O with Tokio",
        "miette": "Pretty error reporting",
        "sarge": "Sarge-based CLI helpers",
        "full": "All core features",
    }

    key_set = set(feature_keys)

    def implied_features(name: str) -> list[str]:
        raw = features_map.get(name, [])
        if not isinstance(raw, list):
            return []
        implied = []
        for item in raw:
            if not isinstance(item, str):
                continue
            if item in key_set and item != "default":
                implied.append(item)
        return implied

    all_non_default = {k for k in key_set if k != "default"}
    full_includes = set(implied_features("full")) | {"full"} if "full" in all_non_default else set()
    full_missing = sorted(all_non_default - full_includes) if full_includes else []

    rows: list[list[str]] = []
    for name in ordered:
        desc = descriptions.get(name, "Optional feature")
        default_mark = "✓" if name in default_set else ""

        notes = ""
        implied = implied_features(name)
        if name == "full" and full_missing:
            notes = "Excludes " + ", ".join(f"`{x}`" for x in full_missing)
        elif name != "full" and implied:
            notes = "Enables " + ", ".join(f"`{x}`" for x in implied)

        rows.append([f"`{name}`", desc, default_mark, notes])

    header = ["Feature", "Description", "Default", "Notes"]
    if not rows:
        raise ValueError("No features found in Cargo.toml [features] section.")
    widths = [
        max([len(header[i])] + [len(r[i]) for r in rows]) for i in range(len(header))
    ]

    lines = []
    lines.append(
        "| "
        + " | ".join(header[i].ljust(widths[i]) for i in range(len(header)))
        + " |"
    )
    lines.append(
        "| "
        + " | ".join("-" * widths[i] for i in range(len(header)))
        + " |"
    )
    for row in rows:
        lines.append(
            "| "
            + " | ".join(row[i].ljust(widths[i]) for i in range(len(row)))
            + " |"
        )
    return lines


def replace_features_table(readme: str, new_table_lines: list[str]) -> str:
    lines = readme.splitlines(keepends=True)
    heading_idx = None
    for idx, line in enumerate(lines):
        if line.strip() == "## Features":
            heading_idx = idx
            break
    if heading_idx is None:
        raise ValueError("README.md is missing a '## Features' section.")

    table_start = None
    for idx in range(heading_idx, len(lines)):
        if lines[idx].lstrip().startswith("| Feature"):
            table_start = idx
            break
    if table_start is None:
        raise ValueError("README.md is missing a features table under '## Features'.")

    table_end = table_start
    while table_end < len(lines) and lines[table_end].lstrip().startswith("|"):
        table_end += 1

    replacement = [line + "\n" for line in new_table_lines]
    lines[table_start:table_end] = replacement
    return "".join(lines)


def replace_readme_versions(readme: str, version: str) -> tuple[str, int]:
    count = 0

    def bump_inline(match: re.Match[str]) -> str:
        nonlocal count
        count += 1
        return f"{match.group(1)}{version}{match.group(3)}"

    inline_pat = re.compile(
        r'(multiio\s*=\s*\{\s*version\s*=\s*\\?")'
        r"([0-9]+\.[0-9]+(?:\.[0-9]+)?(?:-[0-9A-Za-z.-]+)?)"
        r'(\\?")',
    )
    readme = inline_pat.sub(bump_inline, readme)

    def bump_plain(match: re.Match[str]) -> str:
        nonlocal count
        count += 1
        return f"{match.group(1)}{version}{match.group(3)}"

    plain_pat = re.compile(
        r'(multiio\s*=\s*\\?")'
        r"([0-9]+\.[0-9]+(?:\.[0-9]+)?(?:-[0-9A-Za-z.-]+)?)"
        r'(\\?")',
    )
    readme = plain_pat.sub(bump_plain, readme)
    return readme, count


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Sync README dependency version snippets and features matrix table."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root (defaults to the parent of scripts/).",
    )
    parser.add_argument(
        "--version",
        type=str,
        default=None,
        help="Version to sync into README snippets (defaults to Cargo.toml [package].version).",
    )
    parser.add_argument(
        "--features-only",
        action="store_true",
        help="Only update the README features table (do not touch version snippets).",
    )
    args = parser.parse_args(argv)

    root: Path = args.root
    manifest_path = root / "Cargo.toml"
    readme_path = root / "README.md"

    if not manifest_path.exists():
        print(f"error: missing {manifest_path}", file=sys.stderr)
        return 2
    if not readme_path.exists():
        print(f"error: missing {readme_path}", file=sys.stderr)
        return 2

    manifest_text = manifest_path.read_text(encoding="utf-8")
    feature_keys = parse_feature_keys_from_manifest(manifest_text)

    metadata = run_cargo_metadata(manifest_path)
    pkg = select_root_package(metadata, manifest_path)
    package_version = pkg.get("version")
    if not isinstance(package_version, str) or not package_version:
        print("error: could not read package version from cargo metadata.", file=sys.stderr)
        return 2
    features_map = pkg.get("features")
    if not isinstance(features_map, dict):
        print("error: could not read features map from cargo metadata.", file=sys.stderr)
        return 2

    version = args.version or package_version

    readme = readme_path.read_text(encoding="utf-8")
    changed = False

    if not args.features_only:
        readme, replaced = replace_readme_versions(readme, version)
        if replaced == 0:
            print("info: no README version snippets matched; leaving version unchanged.")
        else:
            print(f"info: updated {replaced} README version snippet(s) to {version}.")
            changed = True

    new_table_lines = generate_features_table(feature_keys, features_map)
    updated = replace_features_table(readme, new_table_lines)
    if updated != readme:
        print("info: updated README features table.")
        readme = updated
        changed = True

    if changed:
        readme_path.write_text(readme, encoding="utf-8")
    else:
        print("info: README already up to date.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
