#!/bin/bash
# Autoresearch benchmark runner
# Outputs: METRIC ops_per_sec=X

set -euo pipefail

# Run the benchmark and capture output
OUTPUT=$(cargo bench --bench autoresearch_metric 2>&1)

# Extract the median time line (the line containing "time:" followed by brackets)
TIME_LINE=$(echo "$OUTPUT" | grep -m 1 "time:")

# Extract the three timing values: min median max
BRACKET_CONTENT=$(echo "$TIME_LINE" | sed -n 's/.*\[\(.*\)\].*/\1/p')
# Format: min_value min_unit median_value median_unit max_value max_unit
# We want the median value and its unit
MEDIAN_VALUE=$(echo "$BRACKET_CONTENT" | awk '{print $3}')
MEDIAN_UNIT=$(echo "$BRACKET_CONTENT" | awk '{print $4}')

# Normalize µs unit to 'us' for consistent parsing
MEDIAN_UNIT=$(echo "$MEDIAN_UNIT" | tr 'µ' 'u')

# Convert to seconds
case "$MEDIAN_UNIT" in
    ns) DURATION=$(echo "$MEDIAN_VALUE * 1e-9" | bc -l) ;;
    us) DURATION=$(echo "$MEDIAN_VALUE * 1e-6" | bc -l) ;;
    ms) DURATION=$(echo "$MEDIAN_VALUE * 1e-3" | bc -l) ;;
    s) DURATION=$MEDIAN_VALUE ;;
    *) echo "Unknown unit: $MEDIAN_UNIT (value: $MEDIAN_VALUE)" >&2; exit 1 ;;
esac

# Compute ops/sec: the benchmark runs 1000 operations per iteration
if echo "$DURATION" | grep -q '^0\(\.[0-9]*\)\?$'; then
    echo "Error: zero or near-zero seconds" >&2
    exit 1
fi
OPS_PER_SEC=$(echo "scale=0; 1000 / $DURATION" | bc)

# Output the metric in parseable format
echo "METRIC ops_per_sec=$OPS_PER_SEC"
