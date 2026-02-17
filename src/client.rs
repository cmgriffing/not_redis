use std::time::Duration;
use crate::storage::StorageEngine;
use crate::storage::config::{MaxMemoryPolicy, StorageConfig};
use crate::types::{ToRedisArgs, FromRedisValue, Value};
use crate::error::{RedisResult, RedisError};
use crate::commands::{Cmd, SetOptions, IntegerReplyOrNoOp, CopyOptions, execute_command};

/// A trait defining Redis-compatible commands for a client.
///
/// This trait provides methods for all common Redis operations including
/// strings, lists, sets, hashes, sorted sets, and server commands.
///
/// # Implementors
///
/// This trait is implemented by [`Client`] and can be implemented by other
/// types that want to provide Redis-compatible command interfaces.
pub trait Commands {
    /// Gets the value of a key.
    ///
    /// Returns `None` if the key doesn't exist or the value is not a string.
    fn get<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Sets the value of a key.
    ///
    /// Returns an OK string on success.
    fn set<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Sets the value of a key with additional options.
    ///
    /// Options include expiration time (EX/PX), and conditional flags (NX/XX).
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

    /// Sets multiple key-value pairs at once.
    ///
    /// This operation is atomic - either all keys are set or none are.
    fn mset<K, V, RV>(&mut self, items: &[(K, V)]) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Gets the values of all specified keys.
    ///
    /// Returns a vector of values in the order of the requested keys.
    fn mget<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Deletes one or more keys.
    ///
    /// Returns the number of keys that were deleted.
    fn del<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Checks if one or more keys exist.
    ///
    /// Returns the number of keys that exist.
    fn exists<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Appends a value to the end of a string.
    ///
    /// Returns the length of the string after the append.
    fn append<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns a substring of the string value.
    ///
    /// # Arguments
    /// * `key` - The key
    /// * `from` - Start index (can be negative)
    /// * `to` - End index (can be negative)
    fn getrange<K, RV>(&mut self, key: K, from: isize, to: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Overwrites part of a string starting at the specified offset.
    ///
    /// Returns the length of the string after modification.
    fn setrange<K, V, RV>(&mut self, key: K, offset: isize, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the length of a string value.
    ///
    /// Returns 0 if the key doesn't exist.
    fn strlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Increments the integer value of a key by the given amount.
    ///
    /// Returns the new value after the increment.
    fn incr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Decrements the integer value of a key by the given amount.
    ///
    /// Returns the new value after the decrement.
    fn decr<K, V, RV>(&mut self, key: K, delta: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Gets the value of a field in a hash.
    ///
    /// Returns `None` if the field or key doesn't exist.
    fn hget<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    /// Gets the values of all specified fields in a hash.
    fn hmget<K, F, RV>(&mut self, key: K, fields: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    /// Sets the value of a field in a hash.
    ///
    /// Returns 1 if the field is new, 0 if the field was updated.
    fn hset<K, F, V, RV>(&mut self, key: K, field: F, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Deletes one or more fields from a hash.
    ///
    /// Returns the number of fields that were deleted.
    fn hdel<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns all fields and values in a hash.
    fn hgetall<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns all field names in a hash.
    fn hkeys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns all values in a hash.
    fn hvals<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the number of fields in a hash.
    fn hlen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Increments the integer value of a field in a hash by the given amount.
    fn hincr<K, F, D, RV>(&mut self, key: K, field: F, delta: D) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        D: ToRedisArgs,
        RV: FromRedisValue;

    /// Checks if a field exists in a hash.
    ///
    /// Returns 1 if the field exists, 0 otherwise.
    fn hexists<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
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

    /// Pushes one or more values to the front (left) of a list.
    ///
    /// Returns the length of the list after the push.
    fn lpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Pushes one or more values to the back (right) of a list.
    ///
    /// Returns the length of the list after the push.
    fn rpush<K, V, RV>(&mut self, key: K, value: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Removes and returns the first (left) element of a list.
    fn lpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Removes and returns the last (right) element of a list.
    fn rpop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the length of a list.
    ///
    /// Returns 0 if the key doesn't exist.
    fn llen<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns a range of elements from a list.
    ///
    /// # Arguments
    /// * `key` - The list key
    /// * `start` - Start index (can be negative)
    /// * `stop` - Stop index (can be negative)
    fn lrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the element at the specified index in a list.
    fn lindex<K, RV>(&mut self, key: K, index: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Adds one or more members to a set.
    ///
    /// Returns the number of members that were added.
    fn sadd<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Removes one or more members from a set.
    ///
    /// Returns the number of members that were removed.
    fn srem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns all members of a set.
    fn smembers<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Checks if a member exists in a set.
    ///
    /// Returns 1 if the member exists, 0 otherwise.
    fn sismember<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the number of members in a set.
    fn scard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Removes and returns one or more random members from a set.
    fn spop<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Adds one or more members to a sorted set, or updates its score.
    ///
    /// # Arguments
    /// * `key` - The sorted set key
    /// * `score` - The score (determines ordering)
    /// * `member` - The member to add
    ///
    /// Returns the number of members added.
    fn zadd<K, S, V, RV>(&mut self, key: K, score: S, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        S: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Removes one or more members from a sorted set.
    ///
    /// Returns the number of members removed.
    fn zrem<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns a range of members in a sorted set by index (score order).
    ///
    /// # Arguments
    /// * `key` - The sorted set key
    /// * `start` - Start index (can be negative)
    /// * `stop` - Stop index (can be negative)
    fn zrange<K, RV>(&mut self, key: K, start: isize, stop: isize) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns a range of members in a sorted set by score.
    ///
    /// # Arguments
    /// * `key` - The sorted set key
    /// * `min` - Minimum score (inclusive by default)
    /// * `max` - Maximum score (inclusive by default)
    fn zrangebyscore<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the number of members in a sorted set.
    fn zcard<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the score of a member in a sorted set.
    fn zscore<K, V, RV>(&mut self, key: K, member: V) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the number of members in a sorted set within a score range.
    fn zcount<K, RV>(&mut self, key: K, min: &str, max: &str) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Sets a key's time-to-live in seconds.
    ///
    /// Returns `true` if the timeout was set, `false` if the key doesn't exist.
    fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Sets the expiration on a key by timestamp (Unix timestamp in seconds).
    ///
    /// Returns `true` if the timeout was set, `false` if the key doesn't exist.
    fn expire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Sets a key's time-to-live in milliseconds.
    ///
    /// Returns `true` if the timeout was set, `false` if the key doesn't exist.
    fn pexpire<K>(&mut self, key: K, ms: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Sets the expiration on a key by timestamp (Unix timestamp in milliseconds).
    ///
    /// Returns `true` if the timeout was set, `false` if the key doesn't exist.
    fn pexpire_at<K>(&mut self, key: K, ts: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Returns the time-to-live of a key in seconds.
    ///
    /// Returns -1 if the key exists but has no expiration.
    /// Returns -2 if the key doesn't exist.
    fn ttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs;

    /// Returns the time-to-live of a key in milliseconds.
    ///
    /// Returns -1 if the key exists but has no expiration.
    /// Returns -2 if the key doesn't exist.
    fn pttl<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs;

    /// Removes the expiration from a key.
    ///
    /// Returns `true` if the expiration was removed, `false` if the key doesn't exist.
    fn persist<K>(&mut self, key: K) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Returns the expiration timestamp of a key in seconds.
    ///
    /// Returns -1 if the key exists but has no expiration.
    /// Returns -2 if the key doesn't exist.
    fn expire_time<K>(&mut self, key: K) -> RedisResult<IntegerReplyOrNoOp>
    where
        K: ToRedisArgs;

    /// Sets or clears the bit at the specified offset in a string value.
    ///
    /// Returns the original bit value at the specified offset.
    fn setbit<K>(&mut self, key: K, offset: usize, value: bool) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Returns the bit value at the specified offset in a string value.
    fn getbit<K>(&mut self, key: K, offset: usize) -> RedisResult<bool>
    where
        K: ToRedisArgs;

    /// Returns the number of set bits in a string value.
    ///
    /// This is also known as "popcount".
    fn bitcount<K>(&mut self, key: K) -> RedisResult<usize>
    where
        K: ToRedisArgs;

    /// Returns the number of set bits in a specified range of a string.
    fn bitcount_range<K>(&mut self, key: K, start: usize, end: usize) -> RedisResult<usize>
    where
        K: ToRedisArgs;

    /// Performs a bitwise AND operation between multiple keys.
    ///
    /// Stores the result in the destination key.
    fn bit_and<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    /// Performs a bitwise OR operation between multiple keys.
    ///
    /// Stores the result in the destination key.
    fn bit_or<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    /// Performs a bitwise XOR operation between multiple keys.
    ///
    /// Stores the result in the destination key.
    fn bit_xor<D, S, RV>(&mut self, dstkey: D, srckeys: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    /// Performs a bitwise NOT operation on a key.
    ///
    /// Stores the result in the destination key.
    fn bit_not<D, S, RV>(&mut self, dstkey: D, srckey: S) -> RedisResult<RV>
    where
        D: ToRedisArgs,
        S: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns all keys matching the given pattern.
    ///
    /// Uses glob-style pattern matching:
    /// - `*` matches any number of characters
    /// - `?` matches exactly one character
    /// - `[abc]` matches any character in the brackets
    fn keys<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Returns the data type of a key's value.
    fn key_type<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Renames a key to a new name.
    ///
    /// If the new key already exists, it will be overwritten.
    fn rename<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        N: ToRedisArgs,
        RV: FromRedisValue;

    /// Renames a key to a new name, only if the new key does not exist.
    ///
    /// Returns 1 if the key was renamed, 0 if the new key already exists.
    fn rename_nx<K, N, RV>(&mut self, key: K, new_key: N) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        N: ToRedisArgs,
        RV: FromRedisValue;

    /// Deletes one or more keys in a non-blocking manner.
    ///
    /// Unlike DEL, the key is unlinked in the background.
    fn unlink<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Copies a key from source to destination.
    ///
    /// Can optionally copy to a different database.
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

    /// Pings the server.
    ///
    /// Returns "PONG".
    fn ping<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    /// Echoes the given message back.
    ///
    /// Returns the message that was passed in.
    fn echo<K, RV>(&mut self, msg: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue;

    /// Removes all keys from the current database.
    ///
    /// Returns "OK".
    fn flushdb<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    /// Removes all keys from all databases.
    ///
    /// Returns "OK".
    fn flushall<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    /// Returns the number of keys in the current database.
    fn dbsize<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    /// Returns the UNIX timestamp of the last successful save.
    fn lastsave<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;

    /// Returns the current server time.
    ///
    /// Returns a two-element array: [seconds, microseconds].
    fn time<RV>(&mut self) -> RedisResult<RV>
    where
        RV: FromRedisValue;
}

/// A Redis client for executing commands against an in-memory store.
///
/// The client provides methods for all common Redis operations including
/// strings, lists, sets, hashes, sorted sets, and server commands.
///
/// # Example
///
/// ```rust,no_run
/// use not_redis::Commands;
/// use not_redis::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = Client::new();
///     client.start().await;
///     
///     client.set("key", "value").await?;
///     let value: String = client.get("key").await?;
///     
///     Ok(())
/// }
/// ```
pub struct Client {
    storage: StorageEngine,
}

impl Client {
    /// Creates a new Client with a fresh in-memory storage engine.
    pub fn new() -> Self {
        let storage = StorageEngine::new(100);
        Self { storage }
    }

    /// Creates a new Client with an existing storage engine.
    ///
    /// This allows sharing a storage engine between multiple clients.
    pub fn with_storage(storage: StorageEngine) -> Self {
        Self { storage }
    }

    /// Starts the client, initializing the background expiration sweeper.
    ///
    /// This must be called before using the client to ensure expired keys
    /// are properly cleaned up.
    pub async fn start(&self) {
        self.storage.start_expiration_sweeper().await;
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn set_maxmemory(&self, maxmemory: usize) {
        self.storage.set_maxmemory(maxmemory).await;
    }

    pub async fn set_maxmemory_policy(&self, policy: MaxMemoryPolicy) {
        self.storage.set_maxmemory_policy(policy).await;
    }

    pub async fn get_maxmemory(&self) -> Option<usize> {
        self.storage.get_maxmemory().await
    }

    pub async fn get_maxmemory_policy(&self) -> MaxMemoryPolicy {
        self.storage.get_maxmemory_policy().await
    }

    pub fn current_memory_usage(&self) -> usize {
        self.storage.current_memory_usage()
    }
}

pub struct ClientBuilder {
    config: StorageConfig,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            config: StorageConfig::new(),
        }
    }

    pub fn maxmemory(mut self, maxmemory: usize) -> Self {
        self.config.maxmemory = Some(maxmemory);
        self
    }

    pub fn maxmemory_policy(mut self, policy: MaxMemoryPolicy) -> Self {
        self.config.maxmemory_policy = policy;
        self
    }

    pub fn build(self) -> Client {
        let storage = StorageEngine::with_config(self.config);
        Client { storage }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
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
