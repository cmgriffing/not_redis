# Explanation: Architecture and design

This document explains the internals of not_redis, the design decisions behind it, and the trade-offs compared to Redis.

## Why not_redis exists

Redis is powerful, but it comes with operational overhead: you need a running server, network connectivity, serialization, and connection management. For many applications -- CLI tools, embedded systems, tests, single-process web servers -- the data never leaves the process. not_redis eliminates that overhead by providing a Redis-compatible API that runs entirely in-memory within your application.

## Architecture overview

```
  Client (public API)
    |
    v
  StorageEngine
    |
    +-- DashMap<String, StoredValue>   (concurrent key-value store)
    +-- ExpirationManager              (TTL tracking + background sweep)
    +-- high_water_mark                (compaction trigger)
```

### Client

`Client` is the user-facing API layer. It accepts generic Rust types via `ToRedisArgs` / `FromRedisValue` traits and translates them into operations on `StorageEngine`. All methods are `async` because they run inside a Tokio runtime (required for the expiration sweeper).

### StorageEngine

The core data store. It wraps a `DashMap` -- a concurrent hash map that allows lock-free reads and fine-grained per-shard locking for writes. This is the main reason not_redis is thread-safe without requiring the caller to manage locks.

Each entry in the map is a `StoredValue` containing:
- `data: Arc<RedisData>` -- the actual value, reference-counted for cheap cloning
- `expire_at: Option<Instant>` -- optional expiration timestamp

### ExpirationManager

Manages key TTLs. Internally it keeps a `BTreeMap<Instant, HashSet<String>>` -- a sorted map from expiration times to sets of keys expiring at that time. A background Tokio task (`start_expiration_sweeper`) wakes up every 100 ms, walks the tree up to `Instant::now()`, and removes expired keys from the `DashMap`.

This design means:
- Setting a TTL is O(log n) in the number of distinct expiration times
- The sweeper only processes entries that have actually expired
- Expired keys may linger for up to 100 ms after their TTL (lazy cleanup also happens on read)

### Memory management

`StoredValue.data` is wrapped in `Arc` so that reads can clone cheaply (incrementing a reference count) without copying the underlying data. Mutations use `Arc::make_mut`, which clones the inner data only if other references exist.

The `high_water_mark` tracks the peak number of keys. When the current count drops below 25% of the peak, `DashMap::shrink_to_fit` is called automatically to reclaim memory from deleted entries.

## Data model

not_redis stores data in five structures that map directly to Redis types:

| Redis type | Internal representation | Rust type |
|------------|------------------------|-----------|
| String | Byte vector | `Vec<u8>` |
| List | Double-ended queue | `VecDeque<Vec<u8>>` |
| Set | Hash set | `FxHashSet<Vec<u8>>` |
| Hash | Hash map | `FxHashMap<Vec<u8>, Vec<u8>>` |
| Sorted Set | B-tree map | `BTreeMap<Vec<u8>, f64>` |
| Stream | Vector of entries | `Vec<StreamEntry>` |

`FxHashMap` and `FxHashSet` (from `rustc-hash`) use a faster, non-cryptographic hash function. This is safe here because keys are not adversarially controlled -- they come from your own application code.

## Type conversion system

Two traits bridge the gap between Rust types and Redis values:

**`ToRedisArgs`** converts Rust values into `Value` variants that the engine can store. For example, `"hello"` becomes `Value::String(b"hello".to_vec())` and `42i64` becomes `Value::Int(42)`.

**`FromRedisValue`** converts `Value` back into Rust types. The caller specifies the desired type via turbofish or type annotation:

```rust
let name: String = client.get("key").await?;
let count: i64 = client.get("counter").await?;
```

This mirrors the `redis-rs` crate's approach, making migration between real Redis and not_redis straightforward.

## Concurrency model

not_redis achieves thread safety through `DashMap`, which internally shards data across multiple hash maps, each with its own lock. This means:

- **Reads are lock-free** for most access patterns
- **Writes lock only the affected shard**, not the entire map
- **No global mutex** -- multiple threads can read and write concurrently as long as they touch different shards

The `Client` struct itself requires `&mut self` for operations (to match the redis-rs API ergonomics), so sharing across tasks requires wrapping in `Arc<Mutex<Client>>` or creating multiple clients from the same `StorageEngine`.

## Performance characteristics

not_redis is 100-1000x faster than Redis for in-process workloads. The performance gap comes from:

1. **No network round-trip**: A Redis GET requires a TCP send, kernel context switch, Redis processing, TCP receive, and deserialization. not_redis is a direct function call.

2. **No serialization**: Redis uses the RESP protocol over the wire. not_redis stores native Rust types and converts only at the API boundary.

3. **No connection pool**: Redis clients maintain connection pools with health checks and reconnection logic. not_redis has none of this overhead.

4. **CPU cache locality**: Data lives in the same process heap, staying hot in L1/L2 cache. Redis data lives in a separate process (or container), requiring cache-cold memory access after each context switch.

### When Redis is the better choice

not_redis is not a replacement for Redis. Use Redis when you need:

- **Persistence**: not_redis is purely in-memory; data is lost on process exit
- **Multi-process sharing**: not_redis has no networking layer; data is only accessible within the process
- **Clustering / replication**: not_redis is single-node only
- **Pub/sub**: not implemented
- **Lua scripting**: not implemented
- **Transactions**: MULTI/EXEC is not implemented
- **Large datasets**: not_redis is bounded by your process's heap; Redis can use dedicated memory

## Comparison with similar crates

| Crate | Approach | Trade-off |
|-------|----------|-----------|
| `redis-rs` | Client for a Redis server | Full feature set, but requires a running server |
| `mini-redis` | Educational Redis implementation | Networking included, but not production-grade |
| `dashmap` | Concurrent hash map | Raw data structure; no Redis semantics, TTL, or type system |
| **not_redis** | In-process Redis-compatible library | Redis API without networking; no persistence or clustering |

## Design decisions

**Why `async` for everything?** The expiration sweeper requires Tokio, and matching the redis-rs async API makes it easier to swap between real Redis and not_redis. The async overhead for in-memory operations is negligible.

**Why `DashMap` over `RwLock<HashMap>`?** DashMap's sharded locking provides much better throughput under concurrent writes. A global `RwLock` would serialize all write operations.

**Why `Arc<RedisData>` instead of plain `RedisData`?** Reads are far more common than writes. `Arc` makes reads cheap (clone = reference count increment) at the cost of slightly more expensive writes (copy-on-write via `Arc::make_mut`).

**Why `FxHashMap` over `HashMap`?** The default `HashMap` uses SipHash for DoS resistance. Since not_redis keys come from application code (not untrusted input), the faster FxHash is safe and measurably improves throughput.

## See also

- [Tutorial](tutorial.md) -- learn not_redis step by step
- [How-to guides](how-to.md) -- task-oriented recipes
- [API reference](reference.md) -- all methods and types
