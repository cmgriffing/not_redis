use criterion::{criterion_group, criterion_main, Criterion};
use not_redis::Client;
use tokio::runtime::Runtime;

fn runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn block_on<T>(rt: &Runtime, fut: impl std::future::Future<Output = T>) -> T {
    rt.block_on(fut)
}

fn set_ops(c: &mut Criterion) {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = set_ops
}
criterion_main!(benches);
