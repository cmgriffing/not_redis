use criterion::{criterion_group, criterion_main, Criterion};

fn get_redis_client() -> redis::Client {
    redis::Client::open("redis://localhost:6379").expect("Failed to connect to Redis")
}

mod single_threaded {
    use super::*;

    pub fn string_set(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/string");

        group.bench_function("set", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("SET")
                    .arg("key")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute SET");
            });
        });

        group.bench_function("get_existing", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("SET")
                    .arg("key")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute SET");
                let _: String = redis::cmd("GET")
                    .arg("key")
                    .query(&mut con)
                    .expect("Failed to execute GET");
            });
        });

        group.bench_function("get_missing", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let result: Option<String> = redis::cmd("GET")
                    .arg("nonexistent")
                    .query(&mut con)
                    .expect("Failed to execute GET");
                let _ = result.unwrap_or_default();
            });
        });

        group.finish();
    }

    pub fn hash_set(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/hash");

        group.bench_function("hset", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("HSET")
                    .arg("myhash")
                    .arg("field")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute HSET");
            });
        });

        group.bench_function("hget_existing", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("HSET")
                    .arg("myhash")
                    .arg("field")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute HSET");
                let _: String = redis::cmd("HGET")
                    .arg("myhash")
                    .arg("field")
                    .query(&mut con)
                    .expect("Failed to execute HGET");
            });
        });

        group.bench_function("hget_missing", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let result: Option<String> = redis::cmd("HGET")
                    .arg("myhash")
                    .arg("nonexistent")
                    .query(&mut con)
                    .expect("Failed to execute HGET");
                let _ = result.unwrap_or_default();
            });
        });

        group.finish();
    }

    pub fn list_ops(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/list");

        group.bench_function("lpush", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("LPUSH")
                    .arg("mylist")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute LPUSH");
            });
        });

        group.bench_function("rpush", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("RPUSH")
                    .arg("mylist")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute RPUSH");
            });
        });

        group.bench_function("llen", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("LPUSH")
                    .arg("mylist")
                    .arg("value")
                    .query(&mut con)
                    .expect("Failed to execute LPUSH");
                let _: i64 = redis::cmd("LLEN")
                    .arg("mylist")
                    .query(&mut con)
                    .expect("Failed to execute LLEN");
            });
        });

        group.finish();
    }

    pub fn set_ops(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_threaded/set");

        group.bench_function("sadd", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("SADD")
                    .arg("myset")
                    .arg("member")
                    .query(&mut con)
                    .expect("Failed to execute SADD");
            });
        });

        group.bench_function("smembers", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                let _: () = redis::cmd("SADD")
                    .arg("myset")
                    .arg("member")
                    .query(&mut con)
                    .expect("Failed to execute SADD");
                let _: Vec<String> = redis::cmd("SMEMBERS")
                    .arg("myset")
                    .query(&mut con)
                    .expect("Failed to execute SMEMBERS");
            });
        });

        group.finish();
    }
}

mod throughput {
    use super::*;

    pub fn batch_writes(c: &mut Criterion) {
        let mut group = c.benchmark_group("throughput/batch_writes");

        for batch_size in [100, 1000] {
            group.throughput(criterion::Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                criterion::BenchmarkId::from_parameter(batch_size),
                &batch_size,
                |b, &batch_size| {
                    let client = get_redis_client();
                    let mut con = client.get_connection().expect("Failed to get connection");
                    b.iter(move || {
                        for i in 0..batch_size {
                            let _: () = redis::cmd("SET")
                                .arg(format!("key{}", i))
                                .arg("value")
                                .query(&mut con)
                                .expect("Failed to execute SET");
                        }
                    });
                },
            );
        }
        group.finish();
    }

    pub fn batch_reads(c: &mut Criterion) {
        let mut group = c.benchmark_group("throughput/batch_reads");

        for batch_size in [100, 1000] {
            group.throughput(criterion::Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                criterion::BenchmarkId::from_parameter(batch_size),
                &batch_size,
                |b, &batch_size| {
                    let client = get_redis_client();
                    let mut con = client.get_connection().expect("Failed to get connection");

                    for i in 0..batch_size {
                        let _: () = redis::cmd("SET")
                            .arg(format!("key{}", i))
                            .arg("value")
                            .query(&mut con)
                            .expect("Failed to execute SET");
                    }

                    b.iter(move || {
                        for i in 0..batch_size {
                            let _: String = redis::cmd("GET")
                                .arg(format!("key{}", i))
                                .query(&mut con)
                                .expect("Failed to execute GET");
                        }
                    });
                },
            );
        }
        group.finish();
    }

    pub fn sequential_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("throughput/sequential");

        group.bench_function("set_get_loop_1000", |b| {
            let client = get_redis_client();
            let mut con = client.get_connection().expect("Failed to get connection");
            b.iter(|| {
                for i in 0..1000 {
                    let _: () = redis::cmd("SET")
                        .arg(format!("key{}", i))
                        .arg("value")
                        .query(&mut con)
                        .expect("Failed to execute SET");
                }
                for i in 0..1000 {
                    let _: String = redis::cmd("GET")
                        .arg(format!("key{}", i))
                        .query(&mut con)
                        .expect("Failed to execute GET");
                }
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
