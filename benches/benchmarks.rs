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

mod single_threaded {
    use super::*;

    pub fn string_set(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/string");

        group.bench_function("set", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    client.set("key", "value").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.bench_function("get_existing", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.set("key", "value"));

            b.iter(|| {
                block_on(&rt, async {
                    let _: String = client.get("key").await.unwrap();
                });
            });
        });

        group.bench_function("get_missing", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    let _: String = client.get("nonexistent").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.finish();
    }

    pub fn hash_set(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/hash");

        group.bench_function("hset", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    client.hset("myhash", "field", "value").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.bench_function("hget_existing", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.hset("myhash", "field", "value"));

            b.iter(|| {
                block_on(&rt, async {
                    let _: String = client.hget("myhash", "field").await.unwrap();
                });
            });
        });

        group.bench_function("hget_missing", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    let _: String = client.hget("myhash", "nonexistent").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.finish();
    }

    pub fn list_ops(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/list");

        group.bench_function("lpush", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    client.lpush("mylist", "value").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.bench_function("rpush", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    client.rpush("mylist", "value").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.bench_function("llen", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.lpush("mylist", "value"));

            b.iter(|| {
                block_on(&rt, async {
                    let _: i64 = client.llen("mylist").await.unwrap();
                });
            });
        });

        group.finish();
    }

    pub fn set_ops(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/set");

        group.bench_function("sadd", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    client.sadd("myset", "member").await.unwrap();
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.bench_function("smembers", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.sadd("myset", "member"));

            b.iter(|| {
                block_on(&rt, async {
                    let _: Vec<String> = client.smembers("myset").await.unwrap();
                });
            });
        });

        group.finish();
    }
}

mod concurrency {
    use super::*;

    pub fn concurrent_string_set_different_keys(c: &mut Criterion) {
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

    pub fn concurrent_string_set_same_key(c: &mut Criterion) {
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

    pub fn concurrent_string_get_different_keys(c: &mut Criterion) {
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

    pub fn concurrent_string_get_same_key(c: &mut Criterion) {
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

    pub fn concurrent_hash_ops(c: &mut Criterion) {
        let mut group = c.benchmark_group("concurrency/hash_ops");

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
                                    client
                                        .hset("myhash", format!("field{}", i), "value")
                                        .await
                                        .unwrap();
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

    pub fn concurrent_mixed_read_write(c: &mut Criterion) {
        let mut group = c.benchmark_group("concurrency/mixed_read_write");

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
                                if i % 2 == 0 {
                                    set.spawn(async move {
                                        let mut client = Client::from_storage(storage);
                                        client.set(format!("key{}", i), "newvalue").await.unwrap();
                                    });
                                } else {
                                    set.spawn(async move {
                                        let mut client = Client::from_storage(storage);
                                        let _: String =
                                            client.get(format!("key{}", i)).await.unwrap();
                                    });
                                }
                            }
                            while set.join_next().await.is_some() {}
                        });
                    });
                },
            );
        }
        group.finish();
    }

    pub fn concurrent_list_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("concurrency/list_operations");

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
                                    client.lpush("mylist", format!("value{}", i)).await.unwrap();
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

    pub fn high_contention_same_key(c: &mut Criterion) {
        let mut group = c.benchmark_group("concurrency/high_contention");

        for num_tasks in [50, 100, 500] {
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
                        client.set("contended_key", "initial").await.unwrap();
                    });

                    b.iter(|| {
                        block_on(&rt, async {
                            let mut set = JoinSet::new();
                            for _ in 0..num_tasks {
                                let storage = storage.clone();
                                set.spawn(async move {
                                    let mut client = Client::from_storage(storage);
                                    let _: String = client.get("contended_key").await.unwrap();
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
}

mod throughput {
    use super::*;

    pub fn batch_writes(c: &mut Criterion) {
        let mut group = c.benchmark_group("throughput/batch_writes");

        for batch_size in [100, 1000, 10000] {
            group.throughput(Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(batch_size),
                &batch_size,
                |b, &batch_size| {
                    let rt = runtime();
                    let mut client = Client::new();
                    block_on(&rt, client.start());
                    let _ = block_on(&rt, client.flushdb());

                    b.iter(move || {
                        block_on(&rt, async {
                            for i in 0..batch_size {
                                client.set(format!("key{}", i), "value").await.unwrap();
                            }
                        });
                        let _ = block_on(&rt, client.flushdb());
                    });
                },
            );
        }
        group.finish();
    }

    pub fn batch_reads(c: &mut Criterion) {
        let mut group = c.benchmark_group("throughput/batch_reads");

        for batch_size in [100, 1000, 10000] {
            group.throughput(Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(batch_size),
                &batch_size,
                |b, &batch_size| {
                    let rt = runtime();
                    let mut client = Client::new();
                    block_on(&rt, client.start());

                    // Pre-populate data
                    for i in 0..batch_size {
                        let _ = block_on(&rt, client.set(format!("key{}", i), "value"));
                    }

                    b.iter(move || {
                        block_on(&rt, async {
                            for i in 0..batch_size {
                                let _: String = client.get(format!("key{}", i)).await.unwrap();
                            }
                        });
                    });
                },
            );
        }
        group.finish();
    }

    pub fn sequential_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("throughput/sequential");

        group.bench_function("set_get_loop_1000", |b| {
            let rt = runtime();
            let mut client = Client::new();
            block_on(&rt, client.start());
            let _ = block_on(&rt, client.flushdb());

            b.iter(|| {
                block_on(&rt, async {
                    for i in 0..1000 {
                        client.set(format!("key{}", i), "value").await.unwrap();
                    }
                    for i in 0..1000 {
                        let _: String = client.get(format!("key{}", i)).await.unwrap();
                    }
                });
                let _ = block_on(&rt, client.flushdb());
            });
        });

        group.finish();
    }
}

fn benchmarks(c: &mut Criterion) {
    single_threaded::string_set(c);
    single_threaded::hash_set(c);
    single_threaded::list_ops(c);
    single_threaded::set_ops(c);

    concurrency::concurrent_string_set_different_keys(c);
    concurrency::concurrent_string_set_same_key(c);
    concurrency::concurrent_string_get_different_keys(c);
    concurrency::concurrent_string_get_same_key(c);
    concurrency::concurrent_hash_ops(c);
    concurrency::concurrent_mixed_read_write(c);
    concurrency::concurrent_list_operations(c);
    concurrency::high_contention_same_key(c);

    throughput::batch_writes(c);
    throughput::batch_reads(c);
    throughput::sequential_operations(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = benchmarks
}
criterion_main!(benches);
