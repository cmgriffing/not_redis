# How-to guides

Task-oriented recipes for common not_redis use cases.

## Cache expensive computations

Store the result of a slow operation and serve it from the cache on subsequent calls:

```rust
use not_redis::Client;

async fn get_user_profile(client: &mut Client, user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cache_key = format!("cache:profile:{}", user_id);

    // Try the cache first
    if let Ok(cached) = client.get::<_, String>(&cache_key).await {
        return Ok(cached);
    }

    // Compute the value (e.g. query a database)
    let profile = format!("Profile data for {}", user_id);

    // Store with a 5-minute TTL
    client.set(&cache_key, profile.as_str()).await?;
    client.expire(&cache_key, 300).await?;

    Ok(profile)
}
```

## Implement a session store

Use string keys with TTLs to manage user sessions:

```rust
use not_redis::Client;

async fn create_session(client: &mut Client, session_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("session:{}", session_id);
    client.set(&key, user_id).await?;
    client.expire(&key, 3600).await?; // 1 hour
    Ok(())
}

async fn get_session(client: &mut Client, session_id: &str) -> Option<String> {
    let key = format!("session:{}", session_id);
    client.get::<_, String>(&key).await.ok()
}

async fn destroy_session(client: &mut Client, session_id: &str) {
    let key = format!("session:{}", session_id);
    let _ = client.del(&key).await;
}
```

## Share state between async tasks

Wrap the `Client` in `Arc<Mutex<>>` to share across Tokio tasks:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use not_redis::Client;

#[tokio::main]
async fn main() {
    let client = Arc::new(Mutex::new(Client::new()));
    client.lock().await.start().await;

    let c1 = client.clone();
    let c2 = client.clone();

    let writer = tokio::spawn(async move {
        let mut c = c1.lock().await;
        c.set("counter", "0").await.unwrap();
    });

    let reader = tokio::spawn(async move {
        let mut c = c2.lock().await;
        let val: String = c.get("counter").await.unwrap_or_default();
        println!("counter = {}", val);
    });

    writer.await.unwrap();
    reader.await.unwrap();
}
```

## Share a storage engine across multiple clients

Use `Client::from_storage` to create multiple client handles backed by the same data:

```rust
use not_redis::{Client, StorageEngine};

let storage = StorageEngine::new();

let mut client_a = Client::from_storage(storage.clone());
let mut client_b = Client::from_storage(storage.clone());

// Only need to start the sweeper once
client_a.start().await;

client_a.set("shared_key", "hello").await?;
let val: String = client_b.get("shared_key").await?;
assert_eq!(val, "hello");
```

## Use streams as an event log

Record application events and query them by range:

```rust
use not_redis::Client;

async fn log_event(client: &mut Client, event_type: &str, details: &str) -> Result<String, Box<dyn std::error::Error>> {
    let id = client.xadd(
        "app:events",
        None, // auto-generate ID
        vec![("type", event_type), ("details", details)],
    ).await?;
    Ok(id)
}

async fn recent_events(client: &mut Client, count: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Read last N entries in reverse order
    let entries: Vec<String> = client.xrevrange("app:events", "+", "-", Some(count)).await?;
    Ok(entries)
}

async fn trim_old_events(client: &mut Client, keep: usize) -> Result<i64, Box<dyn std::error::Error>> {
    let removed = client.xtrim("app:events", keep, false).await?;
    Ok(removed)
}
```

## Build a rate limiter

Use string keys with TTLs as sliding-window counters:

```rust
use not_redis::Client;

async fn is_rate_limited(client: &mut Client, user_id: &str, max_requests: i64, window_secs: i64) -> Result<bool, Box<dyn std::error::Error>> {
    let key = format!("ratelimit:{}", user_id);

    // Check current count
    let count: i64 = client.get::<_, String>(&key).await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if count >= max_requests {
        return Ok(true); // rate limited
    }

    // Increment (simple approach: overwrite with new count)
    client.set(&key, (count + 1).to_string().as_str()).await?;

    // Set TTL only on first request in window
    if count == 0 {
        client.expire(&key, window_secs).await?;
    }

    Ok(false)
}
```

## Track unique visitors with sets

```rust
use not_redis::Client;

async fn track_visitor(client: &mut Client, page: &str, visitor_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("visitors:{}", page);
    client.sadd(&key, visitor_id).await?;
    Ok(())
}

async fn unique_visitor_count(client: &mut Client, page: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let key = format!("visitors:{}", page);
    let members: Vec<String> = client.smembers(&key).await?;
    Ok(members)
}
```

## Store structured records in hashes

```rust
use not_redis::Client;

async fn save_product(client: &mut Client, id: u64, name: &str, price: &str, stock: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("product:{}", id);
    client.hset(&key, "name", name).await?;
    client.hset(&key, "price", price).await?;
    client.hset(&key, "stock", stock).await?;
    Ok(())
}

async fn get_product_field(client: &mut Client, id: u64, field: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = format!("product:{}", id);
    let value: String = client.hget(&key, field).await?;
    Ok(value)
}

async fn get_full_product(client: &mut Client, id: u64) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let key = format!("product:{}", id);
    let fields: Vec<String> = client.hgetall(&key).await?;
    Ok(fields)
}
```

## Clean up: delete keys and flush the database

```rust
// Delete a single key
let deleted: i64 = client.del("mykey").await?;

// Check if a key exists before acting
if client.exists("mykey").await? {
    client.del("mykey").await?;
}

// Remove all keys
let _: String = client.flushdb().await?;
```

## See also

- [Tutorial](tutorial.md) -- step-by-step introduction
- [API reference](reference.md) -- all methods and types
- [Explanation](explanation.md) -- architecture and design rationale
