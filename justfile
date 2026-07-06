#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

log := "warn"

alias b := build
alias t := test
alias c := check
alias l := lint

export JUST_LOG := log

lint:
    cargo clippy --all --all-targets --all-features -- --deny warnings
    cargo fmt --all -- --check
    typos -w .
    taplo fmt .

doc-cov:
    RUSTDOCFLAGS='-Z unstable-options '--show-coverage cargo +nightly doc -Zunstable-options --no-deps

# Run tests
test:
    cargo test --all

# Build the project
build:
    cargo build

# Build the project
build-release:
    cargo build --release

doc:
    cargo doc --no-deps --all-features --workspace

open-doc:
    cargo doc --no-deps --all-features --workspace --open

# Clean the target directory
clean:
    cargo clean

# Check for errors without building (quick dev check)
check:
    cargo check

# bit hacky but this should at least work across shells
# checks if there is a pr open from the current branch and if not opens one for you
# will only happen if lint and test pass and there are not uncommitted changes to tracked files
pr: ci
    gh pr list --head "$(git rev-parse --abbrev-ref HEAD)" --json author --jq ". == []" | grep -q "true"
    git diff-index --quiet HEAD --
    gh pr create --web --fill-first

# Run all quality checks: fmt, lint, check, test
ci:
    just lint
    just check
    just test

download-lkd-objects:
    curl -Lo objects.inv https://docs.kernel.org/objects.inv

[working-directory('benches')]
@compile-benches:
    cargo build --release --bins

run-benchmark: download-lkd-objects compile-benches
    hyperfine -N "python benches/sphinx.py" -n "sphinx (python)" --reference benches/target/release/parser --reference-name "sphinx_inv (rust)"  --export-json timing.json --warmup 50 --min-runs 100 --time-unit millisecond

render-comparison:
    uv run benches/render.py timing.json comparison.webp

benchmark: run-benchmark render-comparison
