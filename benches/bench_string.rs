use criterion::{criterion_group, criterion_main, Criterion};
use not_redis::Client;
use tokio::runtime::Runtime;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

fn string_set(c: &mut Criterion) {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = string_set
}
criterion_main!(benches);
