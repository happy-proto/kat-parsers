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
    just test

# Fast contracts only; the official Ghostty CLI is not required.
test:
    cargo test --workspace

# Opt-in locally; CI supplies a freshly downloaded official binary.
ghostty-check:
    test -n "${GHOSTTY_BIN:-}"
    cargo test --locked -p ghostty-conformance --test oracle -- --ignored --nocapture --test-threads=1
