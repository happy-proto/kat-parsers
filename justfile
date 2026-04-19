set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    just check

generate:
    cargo run -p xtask -- generate

check-generated:
    cargo run -p xtask -- generate
    git diff --exit-code

check:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
