#!/usr/bin/env bash
set -euo pipefail

echo "Running integration tests..."

OUTPUT=$(cargo test --test integration_tests 2>&1)
EXIT_CODE=$?

echo "$OUTPUT"
echo ""

if [ $EXIT_CODE -ne 0 ]; then
    echo "FAIL: Tests exited with code $EXIT_CODE"
    exit 1
fi

# Parse the "test result:" summary line (works on both macOS and Linux)
RESULT_LINE=$(echo "$OUTPUT" | grep "^test result:")

if [ -z "$RESULT_LINE" ]; then
    echo "FAIL: Could not parse test results"
    exit 1
fi

PASSED=$(echo "$RESULT_LINE" | sed -E 's/.*[^0-9]([0-9]+) passed.*/\1/')
FAILED=$(echo "$RESULT_LINE" | sed -E 's/.*[^0-9]([0-9]+) failed.*/\1/')

if [ "$FAILED" != "0" ]; then
    echo "FAIL: $FAILED test(s) failed"
    exit 1
fi

echo "SUCCESS: All $PASSED integration tests passed"
exit 0
