use super::Value;
use crate::error::{RedisError, RedisResult};

pub trait FromRedisValue: Sized {
    fn from_redis_value(v: Value) -> RedisResult<Self>;
}

impl FromRedisValue for String {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::String(s) => Ok(String::from_utf8(s).map_err(|_| RedisError::ParseError)?),
            Value::Int(n) => Ok(n.to_string()),
            Value::Okay => Ok("OK".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for Vec<u8> {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::String(s) => Ok(s),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for i64 {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::Int(n) => Ok(n),
            Value::String(s) => String::from_utf8(s)
                .map_err(|_| RedisError::ParseError)?
                .parse::<i64>()
                .map_err(|_| RedisError::ParseError),
            Value::Bool(b) => Ok(if b { 1 } else { 0 }),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for u64 {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        let n: i64 = FromRedisValue::from_redis_value(v)?;
        Ok(n as u64)
    }
}

impl FromRedisValue for isize {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        let n: i64 = FromRedisValue::from_redis_value(v)?;
        Ok(n as isize)
    }
}

impl FromRedisValue for usize {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        let n: i64 = FromRedisValue::from_redis_value(v)?;
        Ok(n as usize)
    }
}

impl FromRedisValue for bool {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::Bool(b) => Ok(b),
            Value::Int(n) => Ok(n != 0),
            Value::String(s) => Ok(!s.is_empty()),
            Value::Null => Ok(false),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for () {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::Null => Ok(()),
            Value::Okay => Ok(()),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl<T: FromRedisValue> FromRedisValue for Option<T> {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::Null => Ok(None),
            v => Ok(Some(T::from_redis_value(v)?)),
        }
    }
}

impl<T: FromRedisValue> FromRedisValue for Vec<T> {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::Array(items) => items.into_iter().map(T::from_redis_value).collect(),
            Value::Null => Ok(Vec::new()),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for Value {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        Ok(v)
    }
}
