use std::time::Duration;
use crate::storage::StorageEngine;
use crate::types::{ToRedisArgs, FromRedisValue, Value};
use crate::error::{RedisResult, RedisError};
use crate::commands::{Cmd, SetOptions, IntegerReplyOrNoOp, CopyOptions, execute_command};

pub trait Commands {
    fn get<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn set<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn set_options<K, V, RV>(
        &mut self,
        key: K,
        value: V,
        options: SetOptions,
    ) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn mset<K, V, RV>(&mut self, items: &[(K, V)]) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn mget<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn del<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn exists<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn append<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn getrange<K, RV>(&mut self, key: K, from: isize, to: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn setrange<K, V, RV>(&mut self, key: K, offset: isize, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn strlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn incr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn decr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
RedisValue;

    fn hget<K, F, RV        RV: From>(&mut self, key: K, -> RedisResult field: F)<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    fn hmget<K, F, RV>(&mut self, key: K, fields: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    fn hset<K, F, V, RV>(&mut self, key: K, field: F, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn hdel<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    fn hgetall<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn hkeys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn hvals<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn hlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn hincr<K, F, D, RV>(&mut self, key: K, field: F, delta: D) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        D: ToRedisArgs,
        RV: FromRedisValue;

    fn hexists<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    fn lpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn rpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn lpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn rpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn llen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn lrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn lindex<K, RV>(&mut self, key: K, index: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn sadd<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn srem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn smembers<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn sismember<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn scard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn spop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn zadd<K, S, V, RV>(&mut self, key: K, score: S, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        S: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn zrem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn zrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn zrangebyscore<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn zcard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn zscore<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    fn zcount<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn expire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn pexpire<K>(&mut self, key: K, ms: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn pexpire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn ttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs;

    fn pttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs;

    fn persist<K>(&mut self, key: K) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn expire_time<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs;

    fn setbit<K>(&mut self, key: K, offset: usize, value: bool) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn getbit<K>(&mut self, key: K, offset: usize) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    fn bitcount<K>(&mut self, key: K) -> RedisResult<usize>
    where
        K: ToRedisArgs;

    fn bitcount_range<K>(&mut self, key: K, start: usize, end: usize) -> RedisResult<usize>
    where
        K: ToRedisArgs;

    fn bit_and<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    fn bit_or<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    fn bit_xor<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    fn bit_not<D, S, RV>(&mut self, dstkey: D, srckey: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    fn keys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn key_type<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn rename<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        N: ToRedisArgs,
        RV: FromRedisValue;

    fn rename_nx<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        N: ToRedisArgs,
        RV: FromRedisValue;

    fn unlink<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn copy<KSrc, KDst, Db, RV>(
        &mut self,
        source: KSrc,
        destination: KDst,
        options: CopyOptions<Db>,
    ) -> RedisResult<RV>
    where
        KSrc: ToRedisArgs,
        KDst: ToRedisArgs,
        Db: ToRedisArgs,
        RV: FromRedisValue;

    fn ping<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    fn echo<K, RV>(&mut self, msg: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    fn flushdb<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    fn flushall<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    fn dbsize<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    fn lastsave<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    fn time<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;
}

pub struct Client {
    storage: StorageEngine,
}

impl Client {
    pub fn new() -> Self {
        let storage = StorageEngine::new(100);
        Self { storage }
    }

    pub fn with_storage(storage: StorageEngine) -> Self {
        Self { storage }
    }

    pub async fn start(&self) {
        self.storage.start_expiration_sweeper().await;
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_command {
    ($method:ident -> $ret:ty) => {
        fn $method<K, RV>(&mut self, key: K) -> RedisResult<RV>
        where
            K: ToRedisArgs,
            RV: FromRedisValue,
        {
            let mut cmd = Cmd::new();
            cmd.arg(stringify!($method).to_uppercase()).arg(key);
            let value = execute_command(&self.storage, &cmd)?;
            RV::from_redis_value(value)
        }
    };
    ($method:ident, $arg1:ident -> $ret:ty) => {
        fn $method<K, $arg1, RV>(&mut self, key: K, $arg1: $arg1) -> RedisResult<RV>
        where
            K: ToRedisArgs,
            $arg1: ToRedisArgs,
            RV: FromRedisValue,
        {
            let mut cmd = Cmd::new();
            cmd.arg(stringify!($method).to_uppercase()).arg(key).arg($arg1);
            let value = execute_command(&self.storage, &cmd)?;
            RV::from_redis_value(value)
        }
    };
}

impl Commands for Client {
    fn get<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("GET").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn set<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SET").arg(key).arg(value);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn set_options<K, V, RV>(
        &mut self,
        key: K,
        value: V,
        options: SetOptions,
    ) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SET").arg(key).arg(value);
        if let Some(ex) = options.ex {
            cmd.arg("EX").arg(ex.as_secs());
        }
        if let Some(px) = options.px {
            cmd.arg("PX").arg(px.as_millis() as i64);
        }
        if options.nx {
            cmd.arg("NX");
        }
        if options.xx {
            cmd.arg("XX");
        }
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn mset<K, V, RV>(&mut self, items: &[(K, V)]) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("MSET");
        for (k, v) in items {
            cmd.arg(k).arg(v);
        }
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn mget<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("MGET").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn del<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("DEL").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn exists<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("EXISTS").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn append<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("APPEND").arg(key).arg(value);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn getrange<K, RV>(&mut self, key: K, from: isize, to: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("GETRANGE").arg(key).arg(from).arg(to);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn setrange<K, V, RV>(&mut self, key: K, offset: isize, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SETRANGE").arg(key).arg(offset).arg(value);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn strlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("STRLEN").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn incr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("INCRBY").arg(key).arg(delta);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn decr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("DECRBY").arg(key).arg(delta);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hget<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HGET").arg(key).arg(field);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hmget<K, F, RV>(&mut self, key: K, fields: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HMGET").arg(key).arg(fields);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hset<K, F, V, RV>(&mut self, key: K, field: F, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HSET").arg(key).arg(field).arg(value);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hdel<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HDEL").arg(key).arg(field);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hgetall<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HGETALL").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hkeys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HKEYS").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hvals<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HVALS").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HLEN").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hincr<K, F, D, RV>(&mut self, key: K, field: F, delta: D) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        D: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HINCRBY").arg(key).arg(field).arg(delta);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn hexists<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("HEXISTS").arg(key).arg(field);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn lpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("LPUSH").arg(key).arg(value);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn rpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("RPUSH").arg(key).arg(value);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn lpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("LPOP").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn rpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("RPOP").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn llen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("LLEN").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn lrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("LRANGE").arg(key).arg(start).arg(stop);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn lindex<K, RV>(&mut self, key: K, index: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("LINDEX").arg(key).arg(index);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn sadd<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SADD").arg(key).arg(member);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn srem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SREM").arg(key).arg(member);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn smembers<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SMEMBERS").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn sismember<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SISMEMBER").arg(key).arg(member);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn scard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SCARD").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn spop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SPOP").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zadd<K, S, V, RV>(&mut self, key: K, score: S, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        S: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZADD").arg(key).arg(score).arg(member);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zrem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZREM").arg(key).arg(member);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZRANGE").arg(key).arg(start).arg(stop);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zrangebyscore<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZRANGEBYSCORE").arg(key).arg(min).arg(max);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zcard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZCARD").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zscore<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZSCORE").arg(key).arg(member);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn zcount<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ZCOUNT").arg(key).arg(min).arg(max);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("EXPIRE").arg(key).arg(seconds);
        let value = execute_command(&self.storage, &cmd)?;
        bool::from_redis_value(value).map_err(|_| RedisError::ParseError)
    }

    fn expire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("EXPIREAT").arg(key).arg(ts);
        let value = execute_command(&self.storage, &cmd)?;
        bool::from_redis_value(value).map_err(|_| RedisError::ParseError)
    }

    fn pexpire<K>(&mut self, key: K, ms: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("PEXPIRE").arg(key).arg(ms);
        let value = execute_command(&self.storage, &cmd)?;
        bool::from_redis_value(value).map_err(|_| RedisError::ParseError)
    }

    fn pexpire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("PEXPIREAT").arg(key).arg(ts);
        let value = execute_command(&self.storage, &cmd)?;
        bool::from_redis_value(value).map_err(|_| RedisError::ParseError)
    }

    fn ttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("TTL").arg(key);
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(IntegerReplyOrNoOp::Integer(val))
    }

    fn pttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("PTTL").arg(key);
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(IntegerReplyOrNoOp::Integer(val))
    }

    fn persist<K>(&mut self, key: K) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("PERSIST").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        bool::from_redis_value(value).map_err(|_| RedisError::ParseError)
    }

    fn expire_time<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("EXPIRETIME").arg(key);
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(IntegerReplyOrNoOp::Integer(val))
    }

    fn setbit<K>(&mut self, key: K, offset: usize, value: bool) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("SETBIT").arg(key).arg(offset).arg(if value { 1i64 } else { 0i64 });
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(val != 0)
    }

    fn getbit<K>(&mut self, key: K, offset: usize) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("GETBIT").arg(key).arg(offset);
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(val != 0)
    }

    fn bitcount<K>(&mut self, key: K) -> RedisResult<usize>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("BITCOUNT").arg(key);
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(val as usize)
    }

    fn bitcount_range<K>(&mut self, key: K, start: usize, end: usize) -> RedisResult<usize>
    where
        K: ToRedisArgs,
    {
        let mut cmd = Cmd::new();
        cmd.arg("BITCOUNT").arg(key).arg(start).arg(end);
        let val: i64 = i64::from_redis_value(execute_command(&self.storage, &cmd)?)
            .map_err(|_| RedisError::ParseError)?;
        Ok(val as usize)
    }

    fn bit_and<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue,
    {
        Err(RedisError::NotSupported)
    }

    fn bit_or<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue,
    {
        Err(RedisError::NotSupported)
    }

    fn bit_xor<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue,
    {
        Err(RedisError::NotSupported)
    }

    fn bit_not<D, S, RV>(&mut self, dstkey: D, srckey: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue,
    {
        Err(RedisError::NotSupported)
    }

    fn keys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("KEYS").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn key_type<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("TYPE").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn rename<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        N: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("RENAME").arg(key).arg(new_key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn rename_nx<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        N: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("RENAMENX").arg(key).arg(new_key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn unlink<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("UNLINK").arg(key);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn copy<KSrc, KDst, Db, RV>(
        &mut self,
        source: KSrc,
        destination: KDst,
        _options: CopyOptions<Db>,
    ) -> RedisResult<RV>
    where
        KSrc: ToRedisArgs,
        KDst: ToRedisArgs,
        Db: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("COPY").arg(source).arg(destination);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn ping<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("PING");
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn echo<K, RV>(&mut self, msg: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("ECHO").arg(msg);
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn flushdb<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("FLUSHDB");
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn flushall<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("FLUSHALL");
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn dbsize<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("DBSIZE");
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn lastsave<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("LASTSAVE");
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }

    fn time<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let mut cmd = Cmd::new();
        cmd.arg("TIME");
        let value = execute_command(&self.storage, &cmd)?;
        RV::from_redis_value(value)
    }
}
