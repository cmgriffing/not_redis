# Autoresearch: Optimize mixed workload throughput

## Objective
Improve the throughput (ops/sec) of the `autoresearch_metric` benchmark, which simulates a realistic mixed workload:
- 40% string SET operations
- 40% string GET operations (on existing keys, 100 key pool)
- 10% hash HSET operations
- 10% hash HGET operations (on existing fields, 100 field pool)

The benchmark runs 1000 operations per iteration and measures the median time, which is converted to operations per second. Higher is better.

Baseline: **6,538,511 ops/sec** (152.94 µs per 1000 ops)

## Metrics
- **Primary**: `ops_per_sec` (unitless, higher is better) — total operations per second
- **Secondary**: (to be added as needed) — may include per-operation timings, memory usage, etc.

## How to Run
`./autoresearch.sh` — outputs `METRIC ops_per_sec=X` line.

## Files in Scope
- `src/storage/engine.rs` — Core storage engine using DashMap. Likely optimization targets: lock contention, allocation patterns, cache locality.
- `src/storage/memory.rs` — (if exists) Memory management
- `src/client.rs` — Client API layer; potential for reducing async overhead, inlining, or reducing allocations
- `src/lib.rs` — Data structures (RedisData enum, Value type); possible optimization of representation
- `benches/autoresearch_metric.rs` — The benchmark itself; can be tuned for better measurement but must maintain the workload distribution

## Off Limits
- Changing the benchmark workload distribution or semantics (must keep 40/40/10/10 ratio and 1000 ops per iteration)
- Adding new dependencies (no externals)
- Breaking the public API (must remain Redis-compatible)
- Modifying correctness — functionality must remain 100% correct

## Constraints
- The benchmark must continue to measure the same mixed workload
- All existing functionality must work correctly (no breaking changes)
- No new dependencies allowed
- Optimizations should be generally applicable, not hyper-specialized to this exact benchmark pattern

## What's Been Tried
*(To be updated as experiments progress)*

Baseline established on 2025-04-08:
- ops_per_sec: 6,538,511

Initial hypotheses to explore:
- Reduce allocation pressure in hot paths (String/Vec allocations)
- Improve cache locality in DashMap access patterns
- Eliminate unnecessary Arc cloning or reference counting
- Optimize the RedisData enum representation
- Reduce async/await overhead in tight loops
- Batch operations or use more efficient data structures for hash/string operations
