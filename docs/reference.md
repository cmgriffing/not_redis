# API reference

Complete reference for all public types and methods in not_redis v0.5.0.

## Client

The main entry point. All data operations go through `Client`.

### Construction and lifecycle

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new() -> Self` | Create a client with a fresh in-memory store |
| `from_storage` | `fn from_storage(storage: StorageEngine) -> Self` | Create a client sharing an existing storage engine |
| `start` | `async fn start(&self)` | Start the background expiration sweeper (call once before use) |

### String commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `set` | `async fn set<K: Into<String>, V: ToRedisArgs>(&mut self, key: K, value: V) -> RedisResult<()>` | Set a key-value pair |
| `get` | `async fn get<K: Into<String>, RV: FromRedisValue>(&mut self, key: K) -> RedisResult<RV>` | Get a value by key |
| `del` | `async fn del<K: ToRedisArgs>(&mut self, key: K) -> RedisResult<i64>` | Delete a key; returns number deleted (0 or 1) |
| `exists` | `async fn exists<K: ToRedisArgs>(&mut self, key: K) -> RedisResult<bool>` | Check if a key exists (expired keys return false) |

### Expiration commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `expire` | `async fn expire<K: ToRedisArgs>(&mut self, key: K, seconds: i64) -> RedisResult<bool>` | Set TTL in seconds. Negative values delete the key. Returns false if key missing |
| `ttl` | `async fn ttl<K: ToRedisArgs>(&mut self, key: K) -> RedisResult<i64>` | Get remaining TTL: `>= 0` = seconds left, `-1` = no expiry, `-2` = key missing |
| `persist` | `async fn persist(&mut self, key: &str) -> bool` | Remove expiration from a key |

### Hash commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `hset` | `async fn hset<K: Into<String>, F: ToRedisArgs, V: ToRedisArgs>(&mut self, key: K, field: F, value: V) -> RedisResult<i64>` | Set a hash field; returns 1 if new, 0 if updated |
| `hget` | `async fn hget<K: Into<String>, F: ToRedisArgs, RV: FromRedisValue>(&mut self, key: K, field: F) -> RedisResult<RV>` | Get a single hash field |
| `hgetall` | `async fn hgetall<K: ToRedisArgs, RV: FromRedisValue>(&mut self, key: K) -> RedisResult<RV>` | Get all fields and values as a flat list `[field, value, ...]` |
| `hdel` | `async fn hdel<K: ToRedisArgs, F: ToRedisArgs>(&mut self, key: K, field: F) -> RedisResult<i64>` | Delete a hash field; returns 1 if removed, 0 if missing |

### List commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `lpush` | `async fn lpush<K: Into<String>, V: ToRedisArgs>(&mut self, key: K, value: V) -> RedisResult<i64>` | Push to the head (left) of a list; returns new length |
| `rpush` | `async fn rpush<K: Into<String>, V: ToRedisArgs>(&mut self, key: K, value: V) -> RedisResult<i64>` | Push to the tail (right) of a list; returns new length |
| `llen` | `async fn llen<K: ToRedisArgs>(&mut self, key: K) -> RedisResult<i64>` | Get list length |

### Set commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `sadd` | `async fn sadd<K: Into<String>, V: ToRedisArgs>(&mut self, key: K, member: V) -> RedisResult<i64>` | Add a member to a set; returns 1 if new, 0 if exists |
| `smembers` | `async fn smembers<K: ToRedisArgs, RV: FromRedisValue>(&mut self, key: K) -> RedisResult<RV>` | Get all members of a set |

### Stream commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `xadd` | `async fn xadd<K, F, V>(&mut self, key: K, entry_id: Option<&str>, values: Vec<(F, V)>) -> RedisResult<String>` | Append an entry; returns entry ID. Pass `None` for auto-generated ID |
| `xlen` | `async fn xlen<K: ToRedisArgs>(&mut self, key: K) -> RedisResult<i64>` | Get the number of entries in a stream |
| `xtrim` | `async fn xtrim<K: ToRedisArgs>(&mut self, key: K, maxlen: usize, approximate: bool) -> RedisResult<i64>` | Trim stream to `maxlen` entries; returns count removed |
| `xdel` | `async fn xdel<K: ToRedisArgs>(&mut self, key: K, entry_ids: Vec<&str>) -> RedisResult<i64>` | Delete entries by ID; returns count deleted |
| `xrange` | `async fn xrange<K, RV>(&mut self, key: K, start: &str, end: &str, count: Option<usize>) -> RedisResult<RV>` | Query entries in ID order. Use `"-"` / `"+"` for min/max |
| `xrevrange` | `async fn xrevrange<K, RV>(&mut self, key: K, start: &str, end: &str, count: Option<usize>) -> RedisResult<RV>` | Query entries in reverse ID order. Use `"+"` / `"-"` for max/min |

### Utility commands

| Method | Signature | Description |
|--------|-----------|-------------|
| `ping` | `async fn ping(&mut self) -> RedisResult<String>` | Returns `"PONG"` |
| `echo` | `async fn echo<K: ToRedisArgs>(&mut self, msg: K) -> RedisResult<String>` | Returns the given message |
| `dbsize` | `async fn dbsize(&mut self) -> RedisResult<i64>` | Returns total number of keys |
| `flushdb` | `async fn flushdb(&mut self) -> RedisResult<String>` | Removes all keys; returns `"OK"` |

---

## StorageEngine

The underlying concurrent data store. Most users interact through `Client`, but `StorageEngine` is public for advanced use cases like sharing state.

| Method | Description |
|--------|-------------|
| `new() -> Self` | Create a new engine (100 ms sweep interval) |
| `start_expiration_sweeper(&self)` | Spawn the background Tokio task for expiry cleanup |
| `set(key, value, expire_at)` | Store a `RedisData` value with optional `Instant` expiry |
| `get(key) -> Option<StoredValue>` | Retrieve a stored value |
| `remove(key) -> bool` | Delete a key |
| `exists(key) -> bool` | Check key existence |
| `len() -> usize` | Number of keys |
| `is_empty() -> bool` | Whether the store is empty |
| `flush()` | Clear all data |
| `set_expiry(key, duration) -> bool` | Set TTL on an existing key |
| `persist(key) -> bool` | Remove TTL from a key |
| `ttl(key) -> Option<Duration>` | Get remaining TTL as `Duration` |
| `ttl_query(key) -> i64` | Get TTL in Redis-compatible format (-2/-1/seconds) |
| `compact()` | Shrink internal allocations to fit current data |
| `xadd`, `xlen`, `xtrim`, `xdel`, `xrange`, `xrevrange` | Low-level stream operations |

---

## Types

### Value

RESP-compatible value type returned from operations:

```rust
pub enum Value {
    Null,
    Int(i64),
    String(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Bool(bool),
    Okay,
}
```

### RedisData

Internal storage representation:

```rust
pub enum RedisData {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(FxHashSet<Vec<u8>>),
    Hash(FxHashMap<Vec<u8>, Vec<u8>>),
    ZSet(BTreeMap<Vec<u8>, f64>),
    Stream(Vec<StreamEntry>),
}
```

### StoredValue

A value with optional expiration metadata:

```rust
pub struct StoredValue {
    pub data: Arc<RedisData>,
    pub expire_at: Option<Instant>,
}
```

### StreamEntry

A single stream entry (ID + field-value pairs):

```rust
pub type StreamEntry = (Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>);
```

### RedisError

```rust
pub enum RedisError {
    ParseError,        // Value could not be parsed into the requested type
    NoSuchKey(String), // Key does not exist
    WrongType,         // Operation against a key holding the wrong data type
    NotSupported,      // Command is not implemented
    Unknown(String),   // Catch-all
}
```

### RedisResult

```rust
pub type RedisResult<T> = Result<T, RedisError>;
```

---

## Traits

### ToRedisArgs

Converts Rust types into Redis command arguments. Implemented for:

`String`, `&str`, `Vec<u8>`, `&[u8]`, `i64`, `u64`, `isize`, `usize`, `bool`, `Option<T>`, `Vec<T>`, and tuples up to 8 elements.

### FromRedisValue

Converts `Value` back into Rust types. Implemented for:

`String`, `Vec<u8>`, `i64`, `u64`, `isize`, `usize`, `bool`, `()`, `Option<T>`, `Vec<T>`, `Value`.

---

## See also

- [Tutorial](tutorial.md) -- learn not_redis step by step
- [How-to guides](how-to.md) -- task-oriented recipes
- [Explanation](explanation.md) -- architecture and design decisions
