set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    just check

generate:
    cargo run -p xtask -- generate

test:
    cargo test --workspace

check:
    cargo run -p xtask -- generate --force
    git diff --exit-code
    cargo test --workspace
