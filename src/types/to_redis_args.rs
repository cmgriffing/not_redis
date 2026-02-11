//! Trait for converting Rust values into Redis command arguments.

use super::Value;

/// A trait for converting values into Redis command arguments.
///
/// This trait is implemented for common Rust types to allow them
/// to be used as keys or values in Redis commands.
///
/// # Implementors
///
/// - `String`, `&str`: Converts to Redis string
/// - `Vec<u8>`, `&[u8]`: Converts to Redis string (raw bytes)
/// - `i64`, `u64`, `isize`, `usize`: Converts to Redis integer
/// - `bool`: Converts to Redis boolean
/// - `Option<T>`: Converts None to null, Some to the inner value
/// - `Vec<T>`: Converts each element
/// - Tuples up to 8 elements: Flattens all elements
pub trait ToRedisArgs {
    fn to_redis_args(&self) -> Vec<Value>;
}

impl ToRedisArgs for String {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![self.as_bytes().to_vec().into()]
    }
}

impl ToRedisArgs for &str {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![self.as_bytes().to_vec().into()]
    }
}

impl ToRedisArgs for Vec<u8> {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![self.clone().into()]
    }
}

impl ToRedisArgs for &[u8] {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![self.to_vec().into()]
    }
}

impl ToRedisArgs for i64 {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![(*self).into()]
    }
}

impl ToRedisArgs for u64 {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![(*self).into()]
    }
}

impl ToRedisArgs for isize {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![(*self as i64).into()]
    }
}

impl ToRedisArgs for usize {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![(*self as i64).into()]
    }
}

impl ToRedisArgs for bool {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![(*self).into()]
    }
}

impl<T: ToRedisArgs> ToRedisArgs for Option<T> {
    fn to_redis_args(&self) -> Vec<Value> {
        match self {
            Some(v) => v.to_redis_args(),
            None => vec![Value::Null],
        }
    }
}

impl<T: ToRedisArgs> ToRedisArgs for Vec<T> {
    fn to_redis_args(&self) -> Vec<Value> {
        self.iter().flat_map(|v| v.to_redis_args()).collect()
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:ident),*) => {
        impl<$($t: ToRedisArgs),*> ToRedisArgs for ($($t,)*) {
            fn to_redis_args(&self) -> Vec<Value> {
                let mut args = Vec::new();
                $(
                    args.extend(self.$idx.to_redis_args());
                )*
                args
            }
        }
    };
}

impl_tuple!();
impl_tuple!(0 T1);
impl_tuple!(0 T1, 1 T2);
impl_tuple!(0 T1, 1 T2, 2 T3);
impl_tuple!(0 T1, 1 T2, 2 T3, 3 T4);
impl_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5);
impl_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6);
impl_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7);
impl_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8);
