#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
IFS=$'\n\t'
umask 0077

echo "==> Running cargo fmt check..."
cargo fmt --manifest-path server_manager/Cargo.toml -- --check

echo "==> Running cargo clippy check..."
cargo clippy --manifest-path server_manager/Cargo.toml --all-targets --all-features -- -D warnings

echo "==> Running cargo tests..."
cargo test --manifest-path server_manager/Cargo.toml --all-targets --all-features

echo "==> Running cargo deny check..."
cargo deny --manifest-path server_manager/Cargo.toml check

echo "==> Running cargo audit check..."
cargo audit --file server_manager/Cargo.lock

echo "==> All verification checks passed successfully!"
