#!/bin/bash
# Custom benchmark runner that outputs parseable metrics for autoresearch

set -e

# Function to run a benchmark and extract ops/sec or ns/op
run_benchmark() {
    local bench_name=$1
    local metric_name=$2
    
    echo "Running benchmark: $bench_name"
    
    # Run the benchmark and capture output
    output=$(cargo bench --bench "$bench_name" 2>&1 | tail -50)
    
    # Extract throughput (ops/s) or time (ns/op) depending on what's available
    if echo "$output" | grep -q "throughput:"; then
        # Extract throughput value (e.g., "throughput: 4.6M ops/s")
        local throughput=$(echo "$output" | grep -oP "throughput:\s+\K[0-9.]+[MK]?")
        local unit=$(echo "$output" | grep -oP "throughput:\s+[0-9.]+[MK]?\s*\K[a-z/]+")
        
        # Convert to numeric value
        local value=$(echo "$throughput" | sed 's/[MK]//')
        local multiplier=1
        if echo "$throughput" | grep -q "M"; then multiplier=1000000
        elif echo "$throughput" | grep -q "K"; then multiplier=1000 fi
        
        local ops_per_sec=$(echo "scale=0; $value * $multiplier / 1" | bc)
        
        echo "METRIC $metric_name=$ops_per_sec"
        echo "$ops_per_sec"
    elif echo "$output" | grep -q "time:"; then
        # Extract time value (e.g., "time: [348.09 ns 350.42 ns 354.25 ns]")
        local median_time=$(echo "$output" | grep -oP "time:\s*\[\K[0-9.]+")
        
        # Convert ns/op to ops/sec
        local ops_per_sec=$(echo "scale=0; 1000000000 / $median_time" | bc)
        
        echo "METRIC $metric_name=$ops_per_sec"
        echo "$ops_per_sec"
    fi
}

# Run key benchmarks
echo "Starting benchmark suite..."

# Single-threaded string operations
run_benchmark "bench_string" "string_set_ops" &
run_benchmark "bench_string" "string_get_existing_ops" &
wait

# Hash operations
run_benchmark "bench_hash" "hash_hset_ops" &
run_benchmark "bench_hash" "hash_hget_ops" &
wait

# List operations
run_benchmark "bench_list" "list_lpush_ops" &
run_benchmark "bench_list" "list_rpush_ops" &
wait

# Set operations
run_benchmark "bench_set" "set_sadd_ops" &
run_benchmark "bench_set" "set_smembers_ops" &
wait

# Throughput benchmarks
run_benchmark "bench_throughput" "batch_writes_100" &
run_benchmark "bench_throughput" "batch_reads_100" &
wait

echo "All benchmarks completed."
