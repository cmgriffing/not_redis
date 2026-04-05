# Tutorial: Getting started with not_redis

This tutorial walks you through using not_redis from scratch. By the end you will have a working Rust program that stores, retrieves, and expires data using an in-memory Redis-compatible store.

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs))
- Basic familiarity with Rust and `async`/`await`

## 1. Create a new project

```bash
cargo new my_cache_app
cd my_cache_app
```

Add not_redis and tokio to `Cargo.toml`:

```toml
[dependencies]
not_redis = "0.5"
tokio = { version = "1", features = ["full"] }
```

## 2. Initialize the client

Replace `src/main.rs` with:

```rust
use not_redis::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new();
    client.start().await; // starts the background expiration sweeper

    println!("not_redis is ready!");
    Ok(())
}
```

Run it:

```bash
cargo run
```

`Client::new()` creates an in-memory store. `client.start().await` launches a background Tokio task that periodically removes expired keys (every 100 ms).

## 3. Store and retrieve strings

```rust
// Set a key
client.set("greeting", "hello world").await?;

// Get it back -- you must specify the return type
let value: String = client.get("greeting").await?;
println!("{}", value); // "hello world"
```

Keys and values can be any type that implements `ToRedisArgs`. Strings, `&str`, `Vec<u8>`, and integers all work.

## 4. Work with hashes

Hashes let you store structured data under a single key:

```rust
client.hset("user:42", "name", "Alice").await?;
client.hset("user:42", "email", "alice@example.com").await?;

let name: String = client.hget("user:42", "name").await?;
println!("Name: {}", name);

// Retrieve all fields and values as a flat vector
let all: Vec<String> = client.hgetall("user:42").await?;
println!("All fields: {:?}", all);
// e.g. ["name", "Alice", "email", "alice@example.com"]
```

## 5. Use lists as queues

```rust
// Push to the left (head) and right (tail)
client.lpush("tasks", "send email").await?;
client.rpush("tasks", "generate report").await?;

let length: i64 = client.llen("tasks").await?;
println!("Pending tasks: {}", length); // 2
```

## 6. Track unique items with sets

```rust
client.sadd("online_users", "alice").await?;
client.sadd("online_users", "bob").await?;
client.sadd("online_users", "alice").await?; // duplicate, ignored

let users: Vec<String> = client.smembers("online_users").await?;
println!("Online: {:?}", users); // ["alice", "bob"]
```

## 7. Add expiration to keys

Keys can be given a time-to-live (TTL) in seconds. Once the TTL elapses, the key is automatically removed.

```rust
client.set("session:abc", "user_42").await?;
client.expire("session:abc", 60).await?; // expires in 60 seconds

let ttl: i64 = client.ttl("session:abc").await?;
println!("Seconds remaining: {}", ttl);

// Remove the expiration (make the key persistent again)
client.persist("session:abc").await;
```

TTL return values follow Redis conventions:
- `>= 0` -- seconds remaining
- `-1` -- key exists but has no expiration
- `-2` -- key does not exist

## 8. Append events with streams

Streams are append-only logs of field-value entries:

```rust
// Add entries (auto-generated IDs)
let id1 = client.xadd("audit", None, vec![("action", "login"), ("user", "alice")]).await?;
let id2 = client.xadd("audit", None, vec![("action", "logout"), ("user", "alice")]).await?;

println!("Entry IDs: {}, {}", id1, id2);

// Count entries
let count: i64 = client.xlen("audit").await?;
println!("Entries: {}", count);

// Read all entries
let entries: Vec<String> = client.xrange("audit", "-", "+", None).await?;
println!("Stream: {:?}", entries);
```

## 9. Handle errors

not_redis methods return `RedisResult<T>`. Common error variants:

```rust
use not_redis::RedisError;

match client.get::<_, String>("nonexistent").await {
    Ok(val) => println!("Got: {}", val),
    Err(RedisError::NoSuchKey(k)) => println!("No such key: {}", k),
    Err(RedisError::WrongType) => println!("Key holds a different data type"),
    Err(e) => println!("Unexpected error: {:?}", e),
}
```

## 10. Check database state

```rust
let exists: bool = client.exists("greeting").await?;
println!("Key exists: {}", exists);

let size: i64 = client.dbsize().await?;
println!("Total keys: {}", size);

// Delete a key
let deleted: i64 = client.del("greeting").await?;
println!("Deleted: {}", deleted);

// Clear everything
let _: String = client.flushdb().await?;
```

## Complete example

```rust
use not_redis::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new();
    client.start().await;

    // Store user profile
    client.hset("user:1", "name", "Alice").await?;
    client.hset("user:1", "role", "admin").await?;

    // Create a session with 30-minute TTL
    client.set("session:xyz", "user:1").await?;
    client.expire("session:xyz", 1800).await?;

    // Track active features
    client.sadd("features:enabled", "dark_mode").await?;
    client.sadd("features:enabled", "notifications").await?;

    // Log an event
    client.xadd("events", None, vec![("type", "user_login"), ("user_id", "1")]).await?;

    // Read back
    let name: String = client.hget("user:1", "name").await?;
    let ttl: i64 = client.ttl("session:xyz").await?;
    let features: Vec<String> = client.smembers("features:enabled").await?;

    println!("User: {}", name);
    println!("Session TTL: {}s", ttl);
    println!("Features: {:?}", features);

    Ok(())
}
```

## Next steps

- [How-to guides](how-to.md) -- solve specific tasks like caching, shared storage, and stream processing
- [API reference](reference.md) -- complete list of methods and types
- [Explanation](explanation.md) -- understand the architecture and design trade-offs
