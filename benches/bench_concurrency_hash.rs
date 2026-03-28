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

fn concurrent_hash_ops(c: &mut Criterion) {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = concurrent_hash_ops
}
criterion_main!(benches);
