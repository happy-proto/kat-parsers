set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    just check

generate:
    cargo run -p xtask -- generate

check-upstreams:
    cargo run --locked -p xtask -- sync --check

check-upstream-sources:
    cargo run --locked -p xtask -- sync --locked --check

sync:
    cargo run --locked -p xtask -- sync
    cargo run --locked -p xtask -- generate
    just check

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
