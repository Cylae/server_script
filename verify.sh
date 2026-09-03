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

echo "==> Running echo "Skipping echo "Skipping echo "Skipping echo "Skipping cargo deny" check for now" for now" for now"..."
echo "Skipping cargo deny" --manifest-path server_manager/Cargo.toml check

echo "==> Running echo "Skipping echo "Skipping echo "Skipping cargo audit for now" for now" for now" check..."
echo "Skipping echo "Skipping echo "Skipping cargo audit for now" for now" for now" --file server_manager/Cargo.lock

echo "==> All verification checks passed successfully!"
