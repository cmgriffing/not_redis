use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use not_redis::Client;
use tokio::runtime::Runtime;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

fn batch_writes(c: &mut Criterion) {
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

fn batch_reads(c: &mut Criterion) {
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

fn sequential_operations(c: &mut Criterion) {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = batch_writes, batch_reads, sequential_operations
}
criterion_main!(benches);
