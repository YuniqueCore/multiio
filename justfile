# ---------------------------------------------------------------------
# Shells
# ---------------------------------------------------------------------
# Unix 默认用 bash（macOS/Linux）
set shell := ["bash", "-cu"]

# Windows 用 PowerShell 7（pwsh）
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]

# ---------------------------------------------------------------------
# Common args
# ---------------------------------------------------------------------
CARGO_TEST_ARGS := "--workspace --all-features --all-targets"
CARGO_DOC_ARGS  := "--workspace --all-features --doc"
CARGO_BUILD_ARGS := "--workspace -F full"

PREK_GIT := "https://github.com/j178/prek"

# show the recipe list
default:
    @just --list

init: init-rust init-prek init-ui

init-rust:
    rustup component add rust-analyzer clippy rustfmt

[unix]
init-prek:
    # Prefer cargo-binstall if present, otherwise cargo install
    if command -v cargo-binstall >/dev/null 2>&1; then \
      cargo binstall -y prek --git {{PREK_GIT}} || cargo install --locked --git {{PREK_GIT}}; \
    else \
      cargo install --locked --git {{PREK_GIT}}; \
    fi

[windows]
init-prek:
    if (Get-Command cargo-binstall -ErrorAction SilentlyContinue) {
        cargo binstall -y prek --git {{PREK_GIT}}
        if ($LASTEXITCODE -ne 0) { cargo install --locked --git {{PREK_GIT}} }
    } else {
        cargo install --locked --git {{PREK_GIT}}
    }

[unix]
init-ui:
    cd web/ui && pnpm install

[windows]
init-ui:
    cd web/ui; pnpm install

# ---------------------------------------------------------------------
# prek
# ---------------------------------------------------------------------
install-prek:
    prek uninstall
    prek install .

prek +ARGS="-a":
    prek run {{ARGS}}

# ---------------------------------------------------------------------
# Rust tests: prefer nextest, fallback to cargo test
# Notes:
# - nextest doesn't support doctests on stable; run `cargo test --doc` separately. :contentReference[oaicite:1]{index=1}
# ---------------------------------------------------------------------
[unix]
cargo-test +EXTRA="":
    if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run {{CARGO_TEST_ARGS}} {{EXTRA}}; \
      cargo test {{CARGO_DOC_ARGS}} {{EXTRA}}; \
    else \
      cargo test {{CARGO_TEST_ARGS}} {{EXTRA}}; \
    fi

[windows]
cargo-test +EXTRA="":
    cargo nextest --version *> $null
    if ($LASTEXITCODE -eq 0) {
        cargo nextest run {{CARGO_TEST_ARGS}} {{EXTRA}}
        cargo test {{CARGO_DOC_ARGS}} {{EXTRA}}
    } else {
        cargo test {{CARGO_TEST_ARGS}} {{EXTRA}}
    }

# ---------------------------------------------------------------------
# e2e
# ---------------------------------------------------------------------
[unix]
e2e:
    cd e2e && uv run pytest

[windows]
e2e:
    cd e2e; uv run pytest

test: cargo-test e2e

# ---------------------------------------------------------------------
# build / lint
# ---------------------------------------------------------------------
build:
    cargo build {{CARGO_BUILD_ARGS}}

happy:
    cargo clippy --fix --allow-dirty --tests -- -D warnings
    cargo fmt --all
    just prek

alias pre-commit := prek
alias lint := happy
alias b := build
alias t := test
alias ct := cargo-test
alias e := e2e
