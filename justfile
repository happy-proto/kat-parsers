set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    just check

generate:
    cargo run -p xtask -- generate

check:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
