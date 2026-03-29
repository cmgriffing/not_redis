# Autoresearch Ideas

## Completed (Applied)

- FxHash for DashMap (faster hashing)
- SmallVec for inline string storage in `RedisData`
- SmallVec for `Value::String` and argument conversion
- Direct DashMap access in hot read paths
- Removed unnecessary `Arc::clone` in several places
- Optimized `key_to_string` to take ownership

 totaled ~9.3% throughput improvement on mixed workload.

## Explored & Discarded

- **Remove `Arc<RedisData>` from `StoredValue`**
  - Would require major API changes to maintain `Send` guarantees.
  - Preliminary attempts led to complex borrow-checker issues and risked read-performance regressions.
  - Not pursued.

- **String interning**
  - For small, inline-capable strings, `SmallVec` is already more efficient (no indirection).
  - Interning adds global state and contention; benefits unclear.

- **Custom allocator / memory pool**
  - Adds complexity for uncertain gains.

- **parking_lot::Mutex**
  - Expiration manager mutex is lightly contended in typical workloads; minimal impact.

## Future Considerations (if profiling indicates)

- Reduce `SmallVec` inline capacity (e.g., 64 → 32) if most strings are very short and memory footprint is critical.
- Optimize integer formatting in `value_to_vec` with `itoa`-style stack formatting (currently allocates a `String`).
- Pre-allocate common collection sizes (e.g., for new `hset` hashes) if workload patterns become predictable.

No further experiments are recommended at this time; the optimization space appears exhausted for general-purpose improvements without compromising API semantics or introducing significant complexity.
