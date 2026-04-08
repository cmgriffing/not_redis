#!/bin/bash
# Correctness checks: run tests and typecheck
set -euo pipefail

# Run tests with minimal output (dot reporter)
cargo test --quiet 2>&1 | tail -50

# Typecheck (cargo check) - ensure it compiles
cargo check --quiet 2>&1 | tail -50
