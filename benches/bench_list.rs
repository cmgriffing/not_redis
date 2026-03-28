use criterion::{criterion_group, criterion_main, Criterion};
use not_redis::Client;
use tokio::runtime::Runtime;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

fn list_ops(c: &mut Criterion) {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = list_ops
}
criterion_main!(benches);
