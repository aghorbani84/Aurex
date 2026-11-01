#!/usr/bin/env sh
# No downloads or native GUI libraries needed for the core workspace.
set -eu
cargo fmt --all --check
cargo fmt --manifest-path apps/aurex-app/Cargo.toml --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked

python3 -m unittest discover -s tests -v
