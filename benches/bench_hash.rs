use criterion::{criterion_group, criterion_main, Criterion};
use not_redis::Client;
use tokio::runtime::Runtime;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

fn hash_set(c: &mut Criterion) {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = hash_set
}
criterion_main!(benches);
