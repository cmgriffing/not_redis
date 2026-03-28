#!/bin/bash
# Extract performance metric from autoresearch_metric benchmark
# Outputs: METRIC ops_per_sec=X

set -e

OUTPUT=$(cargo bench --bench autoresearch_metric 2>&1)

# Debug: save output to file for inspection
# echo "$OUTPUT" > /tmp/bench_output.txt

# Extract the median time line
TIME_LINE=$(echo "$OUTPUT" | grep -m 1 "time:")

# Debug
# echo "Time line: $TIME_LINE" >&2

# Extract the median value (second number in brackets)
# Format could be: "                        time:   [210.27 µs 210.87 µs 211.33 µs]"
# or: "                        time:   [13.908 µs 13.984 µs 14.051 µs]"
# The unit is attached to each number, so we need to parse like: "210.27 µs 210.87 µs 211.33 µs"
# We'll split into tokens
BRACKET_CONTENT=$(echo "$TIME_LINE" | sed -n 's/.*\[\(.*\)\].*/\1/p')
# echo "Bracket content: $BRACKET_CONTENT" >&2

# Split into tokens. Format: min_value min_unit median_value median_unit max_value max_unit
# We want median_value and its unit.
MEDIAN_VALUE=$(echo "$BRACKET_CONTENT" | awk '{print $3}')
MEDIAN_UNIT=$(echo "$BRACKET_CONTENT" | awk '{print $4}')
echo "Median: $MEDIAN_VALUE, Unit (raw): $MEDIAN_UNIT" >&2

# Normalize unit: convert µ to u for microseconds
MEDIAN_UNIT=$(echo "$MEDIAN_UNIT" | tr 'µ' 'u')
echo "Normalized unit: $MEDIAN_UNIT" >&2

# Convert to seconds
case "$MEDIAN_UNIT" in
    ns) DURATION=$(echo "$MEDIAN_VALUE * 1e-9" | bc -l) ;;
    us) DURATION=$(echo "$MEDIAN_VALUE * 1e-6" | bc -l) ;;
    ms) DURATION=$(echo "$MEDIAN_VALUE * 1e-3" | bc -l) ;;
    s) DURATION=$MEDIAN_VALUE ;;
    *) echo "Unknown unit: $MEDIAN_UNIT (value: $MEDIAN_VALUE)"; exit 1 ;;
esac

echo "Seconds: $DURATION" >&2

# Compute ops/sec: 1000 operations per iteration
if echo "$DURATION" | grep -q '^0\(\.[0-9]*\)\?$'; then
    echo "Error: zero or near-zero seconds" >&2
    exit 1
fi
OPS_PER_SEC=$(echo "scale=0; 1000 / $DURATION" | bc)

echo "METRIC ops_per_sec=$OPS_PER_SEC"
echo "Baseline: $OPS_PER_SEC ops/sec (median time per 1000 ops: ${MEDIAN_VALUE}${MEDIAN_UNIT})"
