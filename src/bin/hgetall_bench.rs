#![allow(clippy::let_unit_value)]
use not_redis::{Client, StorageEngine};
use std::time::Instant;

#[tokio::main]
async fn main() {
    // Setup
    let storage = StorageEngine::new();
    storage.start_expiration_sweeper().await;

    // Pre-populate a hash with 100 fields
    let mut client = Client::from_storage(storage.clone());
    for i in 0..100 {
        client
            .hset("bench_hash", format!("field{}", i), format!("value{}", i))
            .await
            .unwrap();
    }

    // Warm-up
    for _ in 0..1000 {
        let _ = client.hgetall::<_, Vec<String>>("bench_hash").await.unwrap();
    }

    // Measure hgetall
    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = client.hgetall::<_, Vec<String>>("bench_hash").await.unwrap();
    }
    let elapsed = start.elapsed();

    // Calculate metrics
    let total_ns = elapsed.as_nanos();
    let avg_ns_per_op = total_ns / iterations as u128;

    // Secondary metrics: measure other operations on same setup
    let mut client2 = Client::from_storage(storage.clone());

    // hset benchmark
    let hset_iterations = 10_000;
    let hset_start = Instant::now();
    for i in 0..hset_iterations {
        let _ = client2.hset("bench_hash", format!("field{}", i), "newvalue").await;
    }
    let hset_elapsed = hset_start.elapsed();
    let hset_ns = hset_elapsed.as_nanos() / hset_iterations as u128;

    // hget benchmark (single field)
    let hget_iterations = 10_000;
    let hget_start = Instant::now();
    for _ in 0..hget_iterations {
        let _: String = client2.hget("bench_hash", "field0").await.unwrap();
    }
    let hget_elapsed = hget_start.elapsed();
    let hget_ns = hget_elapsed.as_nanos() / hget_iterations as u128;

    // string get benchmark
    client2.set("string_key", "string_value").await.unwrap();
    let string_get_iterations = 10_000;
    let string_get_start = Instant::now();
    for _ in 0..string_get_iterations {
        let _: String = client2.get("string_key").await.unwrap();
    }
    let string_get_elapsed = string_get_start.elapsed();
    let string_get_ns = string_get_elapsed.as_nanos() / string_get_iterations as u128;

    // string set benchmark
    let string_set_iterations = 10_000;
    let string_set_start = Instant::now();
    for _ in 0..string_set_iterations {
        let key = format!("key{}", rand::random::<u32>());
        let _ = client2.set(key, "value").await.unwrap();
    }
    let string_set_elapsed = string_set_start.elapsed();
    let string_set_ns = string_set_elapsed.as_nanos() / string_set_iterations as u128;

    // Output structured metrics
    println!("METRIC hgetall_ns={}", avg_ns_per_op);
    println!("METRIC hset_ns={}", hset_ns);
    println!("METRIC hget_ns={}", hget_ns);
    println!("METRIC string_get_ns={}", string_get_ns);
    println!("METRIC string_set_ns={}", string_set_ns);
}
