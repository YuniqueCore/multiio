# Configure PowerShell for Windows
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

# show the recipe list
default:
    @just --list

# install all needed tools (Unix: bash, macOS, Linux)
[unix]
init:
    rustup component add rust-analyzer clippy rustfmt
    cargo binstall prek --git https://github.com/j178/prek 2>/dev/null || cargo install --locked --git https://github.com/j178/prek
    cd web/ui && pnpm install

# install all needed tools (Windows: PowerShell)
[windows]
init:
    rustup component add rust-analyzer clippy rustfmt
    cargo binstall prek --git https://github.com/j178/prek 2>$null; if ($LASTEXITCODE -ne 0) { cargo install --locked --git https://github.com/j178/prek }
    cd web/ui; pnpm install

# install prek (which is the alternative tool of pre-commit)
install-prek:
    prek uninstall
    prek install .

# test schemaui related things
test:
    cargo test --workspace -F full


# build
build:
    cargo build --workspace -F full

# run prek
prek +ARGS="-a":
    prek run {{ARGS}}

# run clippy and rustfmt, then run prek
happy:
    cargo clippy --fix --allow-dirty --tests -- -D warnings
    cargo fmt --all
    just prek

alias pre-commit := prek
alias lint := happy
alias b := build
alias t := test
