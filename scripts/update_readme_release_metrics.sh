#!/usr/bin/env bash
set -euo pipefail

# Keep README release metrics in sync (currently the Features table).
python3 "$(dirname "$0")/sync_readme.py" --features-only
