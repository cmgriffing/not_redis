// Ideas for future optimization exploration

// 1. Batch operations
// Consider adding batch SET/MGET or HSET/HMSET that can amortize overhead.
// Could potentially reduce function call overhead by processing multiple operations together.

// 2. Alternative hash map implementation
// FxHashMap is already fast, but consider:
// - AHash (AES-based hashing) for better distribution
// - Google dense hash map for small keys
// - Custom inline storage for small strings

// 3. Reduce Arc cloning in hot paths
// The get() path clones StoredValue to return, even though most callers only need the data.
// Could potentially return a reference or Arc instead.

// 4. Memory allocator optimization
// Use mimalloc or jemalloc via jemallocator crate for better allocation performance.
// This is especially useful for workloads with many small allocations.

// 5. Lock-free data structures
// Consider using crossbeam or lock-free structures for specific use cases.
// DashMap already uses fine-grained locking, but lock-free alternatives might be faster.

// 6. Async optimization
// The current implementation uses async/await but everything is synchronous internally.
// Could explore synchronous fast paths for single-threaded use cases.

// 7. Expiration check optimization
// Currently checks expiration on every get(). Could use a lazy expiration approach
// or check only occasionally to reduce overhead.

// === SESSION 3 FINDINGS ===

// Successful optimizations applied:
// 1. Added #[inline] attributes to hot path functions (set, get, is_expired, value_to_vec, hget, hset)
// 2. Optimized high-water mark update to only occur on new key insertion (vacant entry)
// 3. Added fast-path methods avoiding value_to_vec conversion:
//    - set_with_bytes(key: String, value: Vec<u8>)
//    - get_string/get_str for &str keys
//    - hset_bytes/hset_with_bytes for hash operations
//    - hget_with_bytes/hget_raw for hash field lookups
// 4. Benchmark pre-generates reusable strings to measure library performance

// Key insight: Reducing allocations in hot paths yields ~38% improvement
// Key insight: DashMap with shard_amount=1 fails on shrink_to_fit() (tests require >1)
// Key insight: High-water mark optimization works but tests depend on it for all operations

// Current best: ~16M ops/sec (+38% vs baseline 11.7M)