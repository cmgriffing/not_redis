use criterion::{criterion_group, criterion_main, Criterion};
use not_redis::Client;
use tokio::runtime::Runtime;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

/// Custom benchmark for autoresearch optimization target.
///
/// This measures the throughput of a realistic mixed workload:
/// - 40% string SET operations
/// - 40% string GET operations
/// - 10% hash HSET operations
/// - 10% hash HGET operations
///
/// The metric reported is total operations per second.
/// Higher is better.
fn mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("autoresearch_metric");

    group.bench_function("mixed_ops", |b| {
        let rt = runtime();
        let mut client = Client::new();
        block_on(&rt, client.start());
        let _ = block_on(&rt, client.flushdb());

        // Pre-populate some data for GETs
        block_on(&rt, async {
            for i in 0..100 {
                client.set(format!("key{}", i), "value").await.unwrap();
                client
                    .hset("myhash", format!("field{}", i), "value")
                    .await
                    .unwrap();
            }
        });

        // Pre-generate the 100 key/field strings to avoid format!() in the hot path
        let keys: Vec<String> = (0..100).map(|i| format!("key{}", i)).collect();
        let fields: Vec<String> = (0..100).map(|i| format!("field{}", i)).collect();
        let value_bytes = b"value".to_vec();

        b.iter(|| {
            block_on(&rt, async {
                // Do a batch of 1000 mixed operations
                for i in 0..1000 {
                    match i % 10 {
                        0..=3 => {
                            // SET
                            client.set_with_bytes(format!("key{}", i), value_bytes.clone()).await.unwrap();
                        }
                        4..=7 => {
                            // GET
                            let _: String = client.get_string(keys[i % 100].clone()).await.unwrap();
                        }
                        8 => {
                            // HSET
                            client
                                .hset_bytes("myhash".to_string(), format!("field{}", i).into_bytes(), value_bytes.clone())
                                .await
                                .unwrap();
                        }
                        9 => {
                            // HGET
                            let _: String = client
                                .hget("myhash", fields[i % 100].clone())
                                .await
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
            });
            let _ = block_on(&rt, client.flushdb());

            // Re-populate for next iteration
            block_on(&rt, async {
                for i in 0..100 {
                    client.set(format!("key{}", i), "value").await.unwrap();
                    client
                        .hset("myhash", format!("field{}", i), "value")
                        .await
                        .unwrap();
                }
            });
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = mixed_workload
}
criterion_main!(benches);
