# Redis-RS API Documentation

This document provides comprehensive context for recreating an API-compatible Redis client library in Rust, based on the redis-rs crate documentation.

## Core Architecture

### Connection Model

The library uses a trait-based abstraction for connections:

```rust
trait ConnectionLike {
    fn req_command(&mut self, cmd: &Cmd) -> RedisResult<Value>;
    fn req_packed_command(&mut self, cmd: &[u8]) -> RedisResult<Value>;
    fn get_db(&self) -> i64;
    fn check_connection(&mut self) -> bool;
    fn is_open(&self) -> bool;
}
```

**Key Types:**
- `Connection` - Stateful TCP connection, bound to a single database
- `Client` - Connector to Redis server, manages connection creation
- `MultiplexedConnection` - Async connection supporting multiple concurrent commands
- `ConnectionManager` - Auto-reconnecting wrapper around MultiplexedConnection
- `PubSub` - Pub/Sub subscription handler
- `Monitor` - Command monitoring receiver

### Error Handling

All operations return `RedisResult<T>` which is an alias for `Result<T, RedisError>`:

```rust
type RedisResult<T> = Result<T, RedisError>;
```

The `RedisError` type encapsulates:
- Connection failures
- Protocol errors
- Command execution errors
- Parse/validation errors

## Command Building

### Cmd Struct (Command Builder)

The `Cmd` struct acts as a builder for Redis commands:

```rust
struct Cmd {
    // Methods
    fn new() -> Cmd
    fn with_capacity(arg_count: usize, size_of_data: usize) -> Cmd
    fn arg<T: ToRedisArgs>(&mut self, arg: T) -> &mut Cmd
    fn cursor_arg(&mut self, cursor: u64) -> &mut Cmd
    fn clear(&mut self)
    fn take(&mut self) -> Self
    
    // Encoding
    fn get_packed_command(&self) -> Vec<u8>
    fn write_packed_command(&self, dst: &mut Vec<u8>)
    
    // Execution (sync)
    fn query<T: FromRedisValue>(&self, con: &mut dyn ConnectionLike) -> RedisResult<T>
    fn exec(&self, con: &mut dyn ConnectionLike) -> RedisResult<()>
    fn iter<T: FromRedisValue>(self, con: &mut dyn ConnectionLike) -> RedisResult<Iter<'_, T>>
    
    // Execution (async)
    fn query_async<T: FromRedisValue>(&self, con: &mut impl ConnectionLike) -> RedisResult<T>
    fn exec_async(&self, con: &mut impl ConnectionLike) -> RedisResult<()>
    fn iter_async<T: FromRedisValue + 'a>(self, con: &'a mut impl AsyncConnection) -> RedisResult<AsyncIter<'a, T>>
    
    // Utility
    fn in_scan_mode(&self) -> bool
    fn args_iter(&self) -> impl ExactSizeIterator<Item = Arg<&[u8]>>
    fn set_no_response(&mut self, nr: bool) -> &mut Cmd
    fn is_no_response(&self) -> bool
    fn set_cache_config(&mut self, command_cache_config: CommandCacheConfig) -> &mut Cmd
}
```

**Usage Example:**
```rust
redis::Cmd::new().arg("SET").arg("my_key").arg(42);
redis::cmd("SET").arg("my_key").arg(42);
```

## Core Traits

### Commands Trait (Generic Return Types)

Provides 271+ methods with generic return types via `FromRedisValue`:

```rust
trait Commands: ConnectionLike {
    // Key-value operations
    fn get<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn set<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    fn set_options<K, V, RV>(&mut self, key: K, value: V, options: SetOptions) -> RedisResult<RV>
    fn mset<K, V, RV>(&mut self, items: &[(K, V)]) -> RedisResult<RV>
    fn mget<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn del<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn exists<K, RV>(&mut self, key: K) -> RedisResult<RV>
    
    // String operations
    fn append<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    fn getrange<K, RV>(&mut self, key: K, from: isize, to: isize) -> RedisResult<RV>
    fn setrange<K, V, RV>(&mut self, key: K, offset: isize, value: V) -> RedisResult<RV>
    fn strlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn incr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    fn decr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    
    // Hash operations
    fn hget<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    fn hmget<K, F, RV>(&mut self, key: K, fields: F) -> RedisResult<RV>
    fn hset<K, F, V, RV>(&mut self, key: K, field: F, value: V) -> RedisResult<RV>
    fn hdel<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    fn hgetall<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn hkeys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn hvals<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn hlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn hincr<K, F, D, RV>(&mut self, key: K, field: F, delta: D) -> RedisResult<RV>
    fn hexists<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    
    // List operations
    fn lpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    fn rpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    fn lpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn rpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn llen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn lrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    fn blpop<K, RV>(&mut self, key: K, timeout: f64) -> RedisResult<RV>
    fn brpop<K, RV>(&mut self, key: K, timeout: f64) -> RedisResult<RV>
    
    // Set operations
    fn sadd<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    fn srem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    fn smembers<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn sismember<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    fn scard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn spop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    
    // Sorted set operations
    fn zadd<K, S, V, RV>(&mut self, key: K, score: S, member: V) -> RedisResult<RV>
    fn zrem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    fn zrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    fn zrangebyscore<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    fn zcard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn zscore<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    fn zcount<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    
    // Expiration operations
    fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    fn expire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    fn pexpire<K>(&mut self, key: K, ms: i64) -> RedisResult<bool>
    fn pexpire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    fn ttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    fn pttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    fn persist<K>(&mut self, key: K) -> RedisResult<bool>
    fn expire_time<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    
    // Bit operations
    fn setbit<K>(&mut self, key: K, offset: usize, value: bool) -> RedisResult<bool>
    fn getbit<K>(&mut self, key: K, offset: usize) -> RedisResult<bool>
    fn bitcount<K>(&mut self, key: K) -> RedisResult<usize>
    fn bitcount_range<K>(&mut self, key: K, start: usize, end: usize) -> RedisResult<usize>
    fn bit_and<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    fn bit_or<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    fn bit_xor<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    fn bit_not<D, S, RV>(&mut self, dstkey: D, srckey: S) -> RedisResult<RV>
    
    // Key operations
    fn keys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn key_type<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn rename<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    fn rename_nx<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    fn unlink<K, RV>(&mut self, key: K) -> RedisResult<RV>
    fn copy<KSrc, KDst, Db, RV>(&mut self, source: KSrc, destination: KDst, options: CopyOptions<Db>) -> RedisResult<RV>
    
    // Server operations
    fn ping<RV>(&mut self) -> RedisResult<RV>
    fn echo<K, RV>(&mut self, msg: K) -> RedisResult<RV>
    fn flushdb<RV>(&mut self) -> RedisResult<RV>
    fn flushall<RV>(&mut self) -> RedisResult<RV>
    fn dbsize<RV>(&mut self) -> RedisResult<RV>
    fn lastsave<RV>(&mut self) -> RedisResult<RV>
    fn time<RV>(&mut self) -> RedisResult<RV>
    
    // Lua scripting
    fn eval<RV>(&mut self, script: &str, keys: &[&str], args: &[&str]) -> RedisResult<RV>
    fn evalsha<RV>(&mut self, sha1: &str, keys: &[&str], args: &[&str]) -> RedisResult<RV>
    fn script_load<RV>(&mut self, script: &str) -> RedisResult<RV>
    fn script_exists<RV>(&mut self, sha1: &[&str]) -> RedisResult<RV>
    fn script_flush<RV>(&mut self) -> RedisResult<RV>
}
```

### TypedCommands Trait (Opinionated Return Types)

Similar to `Commands` but with concrete, typed return values:

```rust
trait TypedCommands {
    fn get<K>(&mut self, key: K) -> RedisResult<Option<String>>
    fn mget<K>(&mut self, key: K) -> RedisResult<Vec<Option<String>>>
    fn keys<K>(&mut self, key: K) -> RedisResult<Vec<String>>
    fn set<K, V>(&mut self, key: K, value: V) -> RedisResult<()>
    fn set_options<K, V>(&mut self, key: K, value: V, options: SetOptions) -> RedisResult<Option<String>>
    fn del<K>(&mut self, key: K) -> RedisResult<usize>
    fn exists<K>(&mut self, key: K) -> RedisResult<bool>
    fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    fn set_nx<K, V>(&mut self, key: K, value: V) -> RedisResult<bool>
    fn incr<K, V>(&mut self, key: K, delta: V) -> RedisResult<isize>
    // ... 273 total methods with typed returns
}
```

### AsyncCommands Trait

Async versions of all Commands methods (271 methods):

```rust
trait AsyncCommands: ConnectionLike + Send {
    fn get<K, RV>(&mut self, key: K) -> RedisFuture<'_, RV>
    fn set<K, V, RV>(&mut self, key: K, value: V) -> RedisFuture<'_, RV>
    // All other Commands methods with async variants
    // Returns RedisFuture<'a, T> instead of RedisResult<T>
}
```

### AsyncTypedCommands Trait

Combines async execution with typed return values (273 methods).

### PubSubCommands Trait

```rust
trait PubSubCommands {
    fn subscribe<P, F, U>(&mut self, patterns: P, func: F) -> RedisResult<U>
    fn psubscribe<P, F, U>(&mut self, patterns: P, func: F) -> RedisResult<U>
}
```

### JsonCommands Trait (Feature: json)

```rust
trait JsonCommands {
    fn json_get<K, P, RV>(&mut self, key: K, path: P) -> RedisResult<RV>
    fn json_set<K, P, V>(&mut self, key: K, path: P, value: V) -> RedisResult<RV>
    fn json_del<K, P>(&mut self, key: K, path: P) -> RedisResult<RV>
    fn json_arr_append<K, P, V>(&mut self, key: K, path: P, value: V) -> RedisResult<RV>
    fn json_arr_insert<K, P, V>(&mut self, key: K, path: P, index: i64, value: V) -> RedisResult<RV>
    fn json_arr_pop<K, P>(&mut self, key: K, path: P, index: i64) -> RedisResult<RV>
    fn json_arr_len<K, P, RV>(&mut self, key: K, path: P) -> RedisResult<RV>
    fn json_obj_keys<K, P, RV>(&mut self, key: K, path: P) -> RedisResult<RV>
    fn json_type<K, P, RV>(&mut self, key: K, path: P) -> RedisResult<RV>
    fn json_num_incr_by<K, P>(&mut self, key: K, path: P, value: i64) -> RedisResult<RV>
}
```

## Type Conversion Traits

### ToRedisArgs Trait

Converts Rust values to Redis arguments:

```rust
trait ToRedisArgs {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: RedisWrite;
    
    fn to_redis_args(&self) -> Vec<Vec<u8>>;
    fn describe_numeric_behavior(&self) -> NumericBehavior;
    fn num_of_args(&self) -> usize;
}
```

**Implemented for:**
- `&str`, `String`
- `bool`
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `f32`, `f64`
- `Vec<u8>` (bytes)
- `&[u8]` (byte slices)
- `BigInt`, `BigUint` (feature: num-bigint)
- `Uuid` (feature: uuid)
- `NonZero*` variants of integers
- `HashMap`, `BTreeMap`, `Vec` (converted to flat arrays)
- Tuples up to 12 elements
- `Option<T>` when T: ToRedisArgs

### FromRedisValue Trait

Converts Redis `Value` to Rust types:

```rust
trait FromRedisValue {
    fn from_redis_value(v: Value) -> Result<Self, ParsingError>;
    fn from_redis_value_ref(v: &Value) -> Result<Self, ParsingError>;
    fn from_redis_value_refs(items: &[Value]) -> Result<Vec<Self>, ParsingError>;
    fn from_redis_values(items: Vec<Value>) -> Result<Vec<Self>, ParsingError>;
    fn from_each_redis_values(items: Vec<Value>) -> Vec<Result<Self, ParsingError>>;
    fn from_byte_slice(vec: &[u8]) -> Option<Vec<Self>>;
    fn from_byte_vec(vec: Vec<u8>) -> Result<Vec<Self>, ParsingError>;
}
```

**Implemented for:**
- `bool`
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `f32`, `f64`
- `String`, `&str`
- `Vec<u8>`, `&[u8]`
- `Vec<T>` for T: FromRedisValue
- `Option<T>` for T: FromRedisValue
- `HashMap<K, V>`, `BTreeMap<K, V>`, `AHashMap<K, V>`
- `HashSet<T>`, `BTreeSet<T>`, `AHashSet<T>`
- `VecDeque<T>` (feature: rb)
- Tuples up to 12 elements
- `()` (unit type)

### Value Enum

Internal representation of Redis response types:

```rust
enum Value {
    Null,
    Int(i64),
    UInt(u64),
    String(Vec<u8>),
    Okay,
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Push(Value, Vec<Value>),
    BStream(Vec<u8>),
}
```

## Client API

### Creating a Client

```rust
impl Client {
    fn open<T: IntoConnectionInfo>(params: T) -> RedisResult<Client>
    
    fn open_with_credentials_provider<T, P>(
        params: T,
        provider: P,
    ) -> RedisResult<Client>
    where
        T: IntoConnectionInfo,
        P: StreamingCredentialsProvider + 'static,
    
    fn with_credentials_provider<P>(self, provider: P) -> Self
    where
        P: StreamingCredentialsProvider + 'static,
    
    fn build_with_tls<C>(
        conn_info: C,
        tls_certs: TlsCertificates,
    ) -> RedisResult<Client>
    where
        C: IntoConnectionInfo,
}

impl Client {
    // Sync connections
    fn get_connection(&self) -> RedisResult<Connection>
    fn get_connection_with_timeout(&self, timeout: Duration) -> RedisResult<Connection>
    fn get_connection_info(&self) -> &ConnectionInfo
    
    // Async connections
    async fn get_multiplexed_async_connection(&self) -> RedisResult<MultiplexedConnection>
    async fn get_multiplexed_async_connection_with_config(
        &self,
        config: &AsyncConnectionConfig,
    ) -> RedisResult<MultiplexedConnection>
    
    async fn get_connection_manager(&self) -> RedisResult<ConnectionManager>
    async fn get_connection_manager_with_config(
        &self,
        config: ConnectionManagerConfig,
    ) -> RedisResult<ConnectionManager>
    
    // Pub/Sub
    async fn get_async_pubsub(&self) -> RedisResult<PubSub>
    async fn get_async_monitor(&self) -> RedisResult<Monitor>
}
```

### Connection URL Format

```plaintext
redis://host:port/db
rediss://host:port/db  (TLS)
```

## Scripting Support

### Script Struct

```rust
struct Script {
    fn new(script: &str) -> Script
    fn arg<T: ToRedisArgs>(&mut self, arg: T) -> &mut Script
    fn key<T: ToRedisArgs>(&mut self, key: T) -> &mut Script
    fn invoke<RV>(&self, con: &mut dyn ConnectionLike) -> RedisResult<RV>
    fn invoke_async<RV>(&self, con: &mut impl AsyncConnection) -> RedisResult<RV>
    fn prepare<RV>(&self, con: &mut dyn ConnectionLike) -> RedisResult<ScriptInvocation>
    fn prepare_async<RV>(&self, con: &mut impl AsyncConnection) -> RedisResult<ScriptInvocation>
}

struct ScriptInvocation {
    fn invoke<RV>(self, con: &mut dyn ConnectionLike) -> RedisResult<RV>
    fn invoke_async<RV>(self, con: &mut impl AsyncConnection) -> RedisResult<RV>
}
```

## Pipeline Support

```rust
struct Pipeline {
    fn new() -> Pipeline
    fn cmd(&mut self, name: &str) -> &mut Pipeline
    fn arg<T: ToRedisArgs>(&mut self, arg: T) -> &mut Pipeline
    fn query(&mut self, con: &mut dyn ConnectionLike) -> RedisResult<Vec<Value>>
    fn query_async(&mut self, con: &mut impl AsyncConnection) -> RedisResult<Vec<Value>>
}
```

## Scan Operations

```rust
struct Iter<'a, T> {
    // Implements Iterator<Item = T>
}

struct AsyncIter<'a, T> {
    // Implements Stream<Item = T>
}

fn scan<K, RV>(&mut self, pattern: K) -> RedisResult<Iter<'_, RV>>
fn scan_match<K, RV>(&mut self, pattern: K) -> RedisResult<Iter<'_, RV>>
fn hscan<K, RV>(&mut self, key: K) -> RedisResult<Iter<'_, RV>>
fn sscan<K, RV>(&mut self, key: K) -> RedisResult<Iter<'_, RV>>
fn zscan<K, RV>(&mut self, key: K) -> RedisResult<Iter<'_, RV>>
```

## Feature Flags

Core features (optional via Cargo.toml):
- `aio` - Async runtime support (tokio-comp, smol-comp)
- `tls-rustls` - TLS support via rustls
- `tokio-comp` - Tokio async runtime
- `smol-comp` - Smol async runtime
- `connection-manager` - Connection manager with auto-reconnect
- `json` - RedisJSON support
- `cache-aio` - Command caching for async
- `num-bigint` - BigInt/BigUint support
- `uuid` - Uuid type support
- `cluster` - Redis Cluster support
- `token-based-authentication` - Token-based auth (Azure, etc.)
- `ahash` - Faster hashing

## Important Design Patterns

### 1. Generic Command Execution
```rust
// Commands trait allows specifying return type
let value: String = con.get("key")?;

// Or use Value for raw response
let value: Value = con.get("key")?;
```

### 2. Connection-Like Abstraction
Any type implementing `ConnectionLike` can use Commands:
- `Connection`
- `MultiplexedConnection`
- `ConnectionManager`
- `Client`

### 3. Typed Commands for Common Patterns
```rust
// TypedCommands returns concrete types
let value: Option<String> = con.get("key")?;

// Commands allows generic return type
let value: Vec<u8> = con.get("key")?;
```

### 4. JSON API Note
RedisJSON results are wrapped in arrays:
> "With RedisJSON commands, you have to note that all results will be wrapped in square brackets (or empty brackets if not found). If you want to deserialize it with e.g. `serde_json` you have to use `Vec<T>` for your output type instead of `T`."

## Stream Operations (Redis Streams)

```rust
struct StreamPendingData { ... }
struct StreamPendingCountReply { ... }
struct StreamPendingId { ... }
struct StreamKey { ... }
struct StreamRangeReply { ... }
struct StreamInfoStreamReply { ... }
struct StreamInfoGroupsReply { ... }
struct StreamInfoGroup { ... }
struct StreamReadReply { ... }
struct StreamTrimOptions { ... }
struct StreamReadOptions { ... }
```

## Vector Operations (RedisVL/Vector Sets)

```rust
struct VAddOptions { ... }
struct VEmbOptions { ... }
struct VSimOptions { ... }
enum VectorSimilaritySearchInput { ... }
enum VectorAddInput { ... }
enum EmbeddingInput { ... }
enum VectorQuantization { ... }
```

## TLS Support

```rust
struct TlsCertificates {
    client_tls: Option<ClientTlsConfig>,
    root_cert: Option<Vec<u8>>,
}

struct ClientTlsConfig {
    client_cert: Vec<u8>,
    client_key: Vec<u8>,
}
```

## Summary

The redis-rs library provides:
1. **Sync and Async APIs** with identical interfaces
2. **Generic type conversion** via `ToRedisArgs` and `FromRedisValue` traits
3. **High-level Commands trait** for ergonomic API
4. **Low-level Cmd builder** for full control
5. **Connection pooling** via MultiplexedConnection
6. **Auto-reconnection** via ConnectionManager
7. **Feature-gated modules** for JSON, scripting, clustering, etc.
8. **Pipeline support** for batch operations
9. **Pub/Sub support** with both RESP2 and RESP3 protocols
