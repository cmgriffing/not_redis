#!/bin/bash
# Pre-commit hook: Run tests before commit

set -e

echo "Running pre-commit tests..."

cd "$(dirname "$0")/.."
cargo fmt --check
cargo clippy -- -D warnings
cargo test --test integration_tests --quiet

echo "Pre-commit checks passed!"
