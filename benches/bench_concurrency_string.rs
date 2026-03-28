use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use not_redis::{Client, StorageEngine};
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

fn concurrent_string_set_different_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency/string_set_different_keys");

    for num_tasks in [10, 50, 100] {
        group.throughput(Throughput::Elements(num_tasks as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                let rt = runtime();
                let storage = StorageEngine::new();
                block_on(&rt, async {
                    storage.start_expiration_sweeper().await;
                });

                b.iter(|| {
                    block_on(&rt, async {
                        let mut set = JoinSet::new();
                        for i in 0..num_tasks {
                            let storage = storage.clone();
                            set.spawn(async move {
                                let mut client = Client::from_storage(storage);
                                client.set(format!("key{}", i), "value").await.unwrap();
                            });
                        }
                        while set.join_next().await.is_some() {}
                        let _ = storage.flush();
                    });
                });
            },
        );
    }
    group.finish();
}

fn concurrent_string_set_same_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency/string_set_same_key");

    for num_tasks in [10, 50, 100] {
        group.throughput(Throughput::Elements(num_tasks as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                let rt = runtime();
                let storage = StorageEngine::new();
                block_on(&rt, async {
                    storage.start_expiration_sweeper().await;
                });

                b.iter(|| {
                    block_on(&rt, async {
                        let mut set = JoinSet::new();
                        for _ in 0..num_tasks {
                            let storage = storage.clone();
                            set.spawn(async move {
                                let mut client = Client::from_storage(storage);
                                client.set("same_key", "value").await.unwrap();
                            });
                        }
                        while set.join_next().await.is_some() {}
                        let _ = storage.flush();
                    });
                });
            },
        );
    }
    group.finish();
}

fn concurrent_string_get_different_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency/string_get_different_keys");

    for num_tasks in [10, 50, 100] {
        group.throughput(Throughput::Elements(num_tasks as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                let rt = runtime();
                let storage = StorageEngine::new();
                block_on(&rt, async {
                    storage.start_expiration_sweeper().await;
                });

                // Pre-populate data
                block_on(&rt, async {
                    let mut client = Client::from_storage(storage.clone());
                    for i in 0..num_tasks {
                        client.set(format!("key{}", i), "value").await.unwrap();
                    }
                });

                b.iter(|| {
                    block_on(&rt, async {
                        let mut set = JoinSet::new();
                        for i in 0..num_tasks {
                            let storage = storage.clone();
                            set.spawn(async move {
                                let mut client = Client::from_storage(storage);
                                let _: String = client.get(format!("key{}", i)).await.unwrap();
                            });
                        }
                        while set.join_next().await.is_some() {}
                    });
                });
            },
        );
    }
    group.finish();
}

fn concurrent_string_get_same_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency/string_get_same_key");

    for num_tasks in [10, 50, 100] {
        group.throughput(Throughput::Elements(num_tasks as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                let rt = runtime();
                let storage = StorageEngine::new();
                block_on(&rt, async {
                    storage.start_expiration_sweeper().await;
                });

                // Pre-populate data
                block_on(&rt, async {
                    let mut client = Client::from_storage(storage.clone());
                    client.set("same_key", "value").await.unwrap();
                });

                b.iter(|| {
                    block_on(&rt, async {
                        let mut set = JoinSet::new();
                        for _ in 0..num_tasks {
                            let storage = storage.clone();
                            set.spawn(async move {
                                let mut client = Client::from_storage(storage);
                                let _: String = client.get("same_key").await.unwrap();
                            });
                        }
                        while set.join_next().await.is_some() {}
                    });
                });
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets =
        concurrent_string_set_different_keys,
        concurrent_string_set_same_key,
        concurrent_string_get_different_keys,
        concurrent_string_get_same_key
}
criterion_main!(benches);
