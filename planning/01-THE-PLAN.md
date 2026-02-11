# not_redis: In-Memory Redis-Compatible Library

An embedded, in-memory Redis-compatible library for Rust applications. Provides Redis-like APIs without networking overhead or external service dependencies.

## Overview

A drop-in replacement for Redis operations that runs entirely within your application's memory. Compatible with redis-rs API patterns.

## Architecture

```
src/
├── lib.rs                           # Library root, re-exports
├── client.rs                        # Client struct (implements Commands)
├── error.rs                         # RedisError type
├── storage/
│   ├── mod.rs
│   ├── engine.rs                    # DashMap<String, StoredValue>
│   └── expire.rs                    # Background TTL sweeper
├── commands/
│   ├── mod.rs                       # Commands trait impl for Client
│   ├── string.rs                    # String command handlers
│   ├── hash.rs                      # Hash command handlers
│   ├── list.rs                      # List command handlers
│   └── set.rs                       # Set command handlers
└── types/
    ├── mod.rs
    ├── value.rs                     # Value enum (matches redis-rs)
    ├── to_redis_args.rs             # ToRedisArgs impls
    └── from_redis_value.rs          # FromRedisValue impls
```

## Features

- **Thread-safe**: Built with `DashMap` for concurrent access
- **TTL Support**: Background task sweeps expired keys
- **Redis-compatible**: Matches redis-rs trait signatures
- **Zero networking**: Pure in-memory operation
- **Tokio async**: Full async/await support

## MVP Command Set

### Strings
- `SET`, `GET`, `DEL`, `EXISTS`
- `SETNX`, `MSET`, `MGET`
- `INCR`, `DECR`, `INCRBY`, `DECRBY`
- `EXPIRE`, `TTL`, `PERSIST`

### Hashes
- `HSET`, `HGET`, `HGETALL`
- `HDEL`, `HEXISTS`, `HMSET`, `HMGET`

### Lists
- `LPUSH`, `RPUSH`, `LPOP`, `RPOP`
- `LLEN`, `LRANGE`

### Sets
- `SADD`, `SREM`, `SMEMBERS`
- `SISMEMBER`, `SCARD`

### Utility
- `PING`, `ECHO`, `DBSIZE`, `FLUSH`

## Usage Example

```rust
use not_redis::{Client, Commands, RedisResult};

#[tokio::main]
async fn main() -> RedisResult<()> {
    let client = Client::new();

    // Strings
    client.set("key", "value").await?;
    let val: String = client.get("key").await?;

    // Hash
    client.hset("user:1", "name", "Alice").await?;
    let name: String = client.hget("user:1", "name").await?;

    // TTL
    client.set("temp", "data").await?;
    client.expire("temp", 60).await?;

    Ok(())
}
```

## Dependencies

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
dashmap = "6.0"
thiserror = "2.0"
```

## Design Decisions

1. **Single namespace**: No MULTI-DB support (SELECT 0, SELECT 1 not needed)
2. **No persistence**: Purely in-memory, no save/load
3. **No Lua scripting**: EVAL/EVALSHA not included
4. **Full trait compatibility**: Implements redis-rs `Commands` trait
5. **DashMap storage**: Thread-safe concurrent HashMap
6. **Background TTL**: `tokio::time::interval` sweeps expired keys

## Roadmap

### v1 (MVP)
- Core string operations with TTL
- Hash, List, Set support
- Basic type conversions

### v2
- Sorted Sets (ZADD, ZRANGE, etc.)
- Pub/Sub support
- More type conversions

### v3
- Stream operations
- Transaction support (MULTI/EXEC)
- Additional Redis commands
