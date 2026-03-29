# Autoresearch Ideas

Potential optimizations not yet pursued:

- Remove `Arc<RedisData>` from `StoredValue` to eliminate one heap allocation per entry and reduce pointer indirection. This would require careful adjustment of all access patterns (both mutable and immutable) and updating tests. Preliminary attempts showed compile errors due to reference pattern mismatches; would need systematic fix.
- Investigate using a custom allocator or memory pool for `SmallVec` spillover to reduce heap fragmentation.
- For keys, consider interning frequently used strings to reduce allocation overhead (may not be applicable for general use).
- Explore using `parking_lot` instead of `std::sync::Mutex` for the expiration manager to reduce contention in concurrent scenarios.
