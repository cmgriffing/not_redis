//! # not_redis
//!
//! A Redis-compatible in-memory data structure library for Rust.
//!
//! not_redis provides Redis-like APIs without the networking overhead, external
//! service dependencies, or operational complexity of running a Redis server.
//!
//! ## Features
//!
//! - **Zero-config**: Embeddable Redis-compatible storage
//! - **Thread-safe**: Concurrent access via Tokio and DashMap
//! - **RESP-compatible**: Data types and command semantics compatible with Redis
//! - **In-process**: No network overhead - runs in your application
//!
//! ## Supported Commands
//!
//! - **Strings**: GET, SET, DEL, EXISTS, EXPIRE, TTL, PERSIST
//! - **Hashes**: HSET, HGET, HGETALL, HDEL
//! - **Lists**: LPUSH, RPUSH, LLEN
//! - **Sets**: SADD, SMEMBERS
//! - **Utilities**: PING, ECHO, DBSIZE, FLUSHDB
//!
//! ## Example
//!
//! ```rust,no_run
//! use not_redis::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::new();
//!     client.start().await;
//!
//!     // String operations
//!     client.set("user:1:name", "Alice").await?;
//!     let name: String = client.get("user:1:name").await?;
//!
//!     // Hash operations
//!     client.hset("user:1", "email", "alice@example.com").await?;
//!     let email: String = client.hget("user:1", "email").await?;
//!
//!     // Expiration
//!     client.expire("user:1", 60).await?;
//!     let ttl: i64 = client.ttl("user:1").await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![allow(clippy::needless_return)]

use dashmap::DashMap;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use smallvec::smallvec;
use std::collections::{BTreeMap, VecDeque};
use std::hash::BuildHasherDefault;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Error type for Redis operations.
///
/// This enum represents the various errors that can occur when
/// interacting with the Redis-like store.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RedisError {
    #[error("Cannot parse value")]
    ParseError,
    #[error("No such key: {0}")]
    NoSuchKey(String),
    #[error("Wrong type operation")]
    WrongType,
    #[error("Not supported command")]
    NotSupported,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// A specialized `Result` type for Redis operations.
pub type RedisResult<T> = Result<T, RedisError>;

/// Represents a value stored in or returned from Redis.
///
/// This enum mirrors the RESP (REdis Serialization Protocol) types
/// supported by Redis.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Value {
    Null,
    Int(i64),
    String(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Bool(bool),
    Okay,
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s.into_bytes())
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.as_bytes().to_vec())
    }
}
impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::String(b)
    }
}
impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

/// Represents a single entry in a Redis stream.
pub type StreamEntry = (Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>);

/// Internal data types stored in the engine.
///
/// These represent the actual data structures that can be stored,
/// as opposed to the RESP protocol [`Value`] types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum RedisData {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(FxHashSet<Vec<u8>>),
    Hash(FxHashMap<Vec<u8>, Vec<u8>>),
    ZSet(BTreeMap<Vec<u8>, f64>),
    Stream(Vec<StreamEntry>),
}

/// A value stored in the storage engine with optional expiration.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct StoredValue {
    pub data: Arc<RedisData>,
    pub expire_at: Option<Instant>,
}

#[allow(missing_docs)]
impl StoredValue {
    pub fn is_expired(&self) -> bool {
        self.expire_at.is_some_and(|at| Instant::now() >= at)
    }
}

#[derive(Clone)]
struct ExpirationManager {
    expirations: Arc<Mutex<BTreeMap<Instant, FxHashSet<String>>>>,
    sweep_interval: Duration,
}

impl ExpirationManager {
    fn new(ms: u64) -> Self {
        Self {
            expirations: Arc::new(Mutex::new(BTreeMap::new())),
            sweep_interval: Duration::from_millis(ms),
        }
    }

    fn schedule(&self, key: String, at: Instant) {
        let mut e = self.expirations.lock().unwrap();
        e.entry(at).or_default().insert(key);
    }

    fn cancel(&self, key: &str) {
        let mut e = self.expirations.lock().unwrap();
        for (_, keys) in e.iter_mut() {
            keys.remove(key);
        }
    }

    fn clear(&self) {
        let mut e = self.expirations.lock().unwrap();
        e.clear();
    }
}

type FxBuildHasher = BuildHasherDefault<FxHasher>;

/// The core storage engine for the Redis-like store.
///
/// Uses a concurrent hash map ([`DashMap`]) for thread-safe access
/// and supports key expiration with a background sweeper task.
#[derive(Clone)]
pub struct StorageEngine {
    data: Arc<DashMap<String, StoredValue, FxBuildHasher>>,
    expiration: ExpirationManager,
    high_water_mark: Arc<AtomicUsize>,
    current_len: Arc<AtomicUsize>,
}

#[allow(missing_docs)]
impl StorageEngine {
    /// Creates a new storage engine with the default sweep interval.
    ///
    /// The sweep interval determines how often expired keys are cleaned up
    /// by the background task when started.
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::with_hasher_and_shard_amount(FxBuildHasher::default(), 2)),
            expiration: ExpirationManager::new(100),
            high_water_mark: Arc::new(AtomicUsize::new(0)),
            current_len: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Starts the background task that periodically sweeps expired keys.
    ///
    /// This spawns a Tokio task that runs indefinitely, removing keys
    /// that have passed their expiration time.
    pub async fn start_expiration_sweeper(&self) {
        let expiration = self.expiration.clone();
        let data = Arc::clone(&self.data);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(expiration.sweep_interval);
            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut e = expiration.expirations.lock().unwrap();
                let expired: Vec<(Instant, Vec<String>)> = e
                    .iter()
                    .filter(|(t, _)| **t <= now)
                    .map(|(t, keys)| (*t, keys.iter().cloned().collect()))
                    .collect();
                for (t, keys) in expired {
                    e.remove(&t);
                    for key in keys {
                        expiration.cancel(&key);
                        data.remove(&key);
                    }
                }
            }
        });
    }

    /// Sets a key-value pair in the storage engine.
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The data to store
    /// * `expire_at` - Optional expiration time
    pub fn set(&self, key: impl Into<String>, value: RedisData, expire_at: Option<Instant>) {
        let key = key.into();
        // If expiration is set, we will need the key later to schedule.
        // Clone it only if needed to avoid unnecessary allocation.
        let key_for_expire = expire_at.as_ref().map(|_| key.clone());

        match self.data.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                // Check if the old entry had an expiration without cloning
                if entry.get().expire_at.is_some() {
                    self.expiration.cancel(entry.key());
                }
                entry.insert(StoredValue {
                    data: Arc::new(value),
                    expire_at,
                });
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(StoredValue {
                    data: Arc::new(value),
                    expire_at,
                });
                // Increment current length counter for new key
                self.current_len.fetch_add(1, Ordering::Relaxed);
            }
        }

        if let (Some(at), Some(key_expire)) = (expire_at, key_for_expire) {
            self.expiration.schedule(key_expire, at);
        }

        // Update high-water mark (cheap atomic load)
        let current_len = self.current_len.load(Ordering::Relaxed);
        self.high_water_mark.fetch_max(current_len, Ordering::Relaxed);
    }

    /// Gets a value from the storage engine by key.
    ///
    /// Returns the stored value if the key exists and has not expired.
    pub fn get(&self, key: &str) -> Option<StoredValue> {
        self.data.get(key).map(|v| v.clone())
    }

    /// Removes a key from the storage engine.
    ///
    /// Returns `true` if the key was present, `false` otherwise.
    /// Also removes any scheduled expiration for the key.
    pub fn remove(&self, key: &str) -> bool {
        self.expiration.cancel(key);
        let removed = self.data.remove(key).is_some();
        if removed {
            self.current_len.fetch_sub(1, Ordering::Relaxed);
            self.maybe_compact();
        }
        removed
    }

    /// Compacts the storage engine by shrinking the DashMap's internal allocations.
    ///
    /// This reclaims memory from removed entries by shrinking each shard's
    /// backing storage to fit only the current entries. The high-water mark
    /// is reset to the current number of entries.
    pub fn compact(&self) {
        self.data.shrink_to_fit();
        self.high_water_mark
            .store(self.current_len.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    fn maybe_compact(&self) {
        let hwm = self.high_water_mark.load(Ordering::Relaxed);
        if hwm == 0 {
            return;
        }
        let current_len = self.current_len.load(Ordering::Relaxed);
        if current_len * 4 < hwm {
            self.compact();
        }
    }

    /// Checks if a key exists in the storage engine.
    ///
    /// Returns `true` if the key exists, `false` otherwise.
    /// Note: This does not check if the key has expired.
    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns the number of keys in the storage engine.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the storage engine contains no keys.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clears all data from the storage engine.
    ///
    /// This removes all keys and their values, and cancels all scheduled expirations.
    pub fn flush(&self) {
        self.data.clear();
        self.expiration.clear();
        self.high_water_mark.store(0, Ordering::Relaxed);
        self.current_len.store(0, Ordering::Relaxed);
    }

    /// Sets an expiration time on an existing key.
    ///
    /// # Arguments
    /// * `key` - The key to set expiration on
    /// * `dur` - The duration until expiration
    ///
    /// Returns `true` if the key exists and expiration was set, `false` otherwise.
    pub fn set_expiry(&self, key: &str, dur: Duration) -> bool {
        if let Some(mut e) = self.data.get_mut(key) {
            let at = Instant::now() + dur;
            e.expire_at = Some(at);
            self.expiration.schedule(key.to_string(), at);
            return true;
        }
        false
    }

    /// Removes the expiration from a key, making it persistent.
    ///
    /// Returns `true` if the key existed and expiration was removed, `false` otherwise.
    pub fn persist(&self, key: &str) -> bool {
        if let Some(mut e) = self.data.get_mut(key) {
            e.expire_at = None;
            self.expiration.cancel(key);
            return true;
        }
        false
    }

    /// Returns the time-to-live remaining for a key.
    ///
    /// Returns `Some(Duration)` if the key has an expiration,
    /// or `None` if the key does not exist or has no expiration.
    pub fn ttl(&self, key: &str) -> Option<Duration> {
        self.data.get(key).and_then(|e| {
            e.expire_at
                .map(|at| at.saturating_duration_since(Instant::now()))
        })
    }

    /// Returns the TTL of a key in seconds, in Redis-compatible format.
    ///
    /// Returns:
    /// - `-1` if the key exists but has no expiration
    /// - `-2` if the key does not exist
    /// - A non-negative value representing seconds until expiration
    pub fn ttl_query(&self, key: &str) -> i64 {
        self.data.get(key).map_or(-2i64, |e| match e.expire_at {
            Some(at) => at.saturating_duration_since(Instant::now()).as_secs() as i64,
            None => -1i64,
        })
    }

    /// Adds an entry to a stream.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `entry_id` - Optional custom entry ID. If None, auto-generates one
    /// * `values` - Field-value pairs to add
    ///
    /// Returns the entry ID if successful, None if key exists but is not a stream.
    pub fn xadd(
        &self,
        key: &str,
        entry_id: Option<&[u8]>,
        values: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Option<Vec<u8>> {
        let new_id = match entry_id {
            Some(id) => id.to_vec(),
            None => self.generate_stream_id(),
        };

        let entry = (new_id.clone(), values);

        if let Some(mut stored) = self.data.get_mut(key) {
            match Arc::make_mut(&mut stored.data) {
                RedisData::Stream(entries) => {
                    entries.push(entry);
                    Some(new_id)
                }
                _ => None,
            }
        } else {
            let entries = vec![entry];
            self.set(key, RedisData::Stream(entries), None);
            Some(new_id)
        }
    }

    /// Returns the number of entries in a stream.
    ///
    /// Returns the length if the key exists and is a stream, None otherwise.
    pub fn xlen(&self, key: &str) -> Option<usize> {
        self.data.get(key).map(|stored| match &*stored.data {
            RedisData::Stream(entries) => entries.len(),
            _ => 0,
        })
    }

    /// Trims a stream to a maximum number of entries.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `maxlen` - Maximum number of entries to keep
    /// * `approximate` - If true, uses approximate trimming (keeps maxlen - 10%)
    ///
    /// Returns the number of entries removed, or None if key is not a stream.
    pub fn xtrim(&self, key: &str, maxlen: usize, approximate: bool) -> Option<usize> {
        if let Some(mut stored) = self.data.get_mut(key) {
            match Arc::make_mut(&mut stored.data) {
                RedisData::Stream(entries) => {
                    let original_len = entries.len();
                    if entries.len() > maxlen {
                        let keep = if approximate {
                            maxlen.saturating_sub(maxlen / 10)
                        } else {
                            maxlen
                        };
                        entries.drain(0..entries.len().saturating_sub(keep));
                        entries.shrink_to_fit();
                    }
                    return Some(original_len.saturating_sub(entries.len()));
                }
                _ => return None,
            }
        }
        None
    }

    /// Deletes entries from a stream.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `entry_ids` - Entry IDs to delete
    ///
    /// Returns the number of entries deleted, or None if key is not a stream.
    pub fn xdel(&self, key: &str, entry_ids: Vec<&[u8]>) -> Option<usize> {
        if let Some(mut stored) = self.data.get_mut(key) {
            match Arc::make_mut(&mut stored.data) {
                RedisData::Stream(entries) => {
                    let original_len = entries.len();
                    entries.retain(|(id, _)| !entry_ids.contains(&id.as_slice()));
                    entries.shrink_to_fit();
                    return Some(original_len.saturating_sub(entries.len()));
                }
                _ => return None,
            }
        }
        None
    }

    /// Returns entries in a stream within a range.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `start` - Start ID (use "-" for beginning)
    /// * `end` - End ID (use "+" for end)
    /// * `count` - Optional maximum number of entries to return
    ///
    /// Returns the entries in the range, or None if key is not a stream.
    pub fn xrange(
        &self,
        key: &str,
        start: &[u8],
        end: &[u8],
        count: Option<usize>,
    ) -> Option<Vec<StreamEntry>> {
        self.data.get(key).map(|stored| match &*stored.data {
            RedisData::Stream(entries) => {
                let mut result: Vec<_> = entries
                    .iter()
                    .filter(|(id, _)| {
                        let ge_start = start == b"-" || id.as_slice() >= start;
                        let le_end = end == b"+" || id.as_slice() <= end;
                        ge_start && le_end
                    })
                    .cloned()
                    .collect();

                if let Some(c) = count {
                    result.truncate(c);
                }
                result
            }
            _ => vec![],
        })
    }

    /// Returns entries in a stream within a range, in reverse order.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `start` - Start ID (use "+" for end)
    /// * `end` - End ID (use "-" for beginning)
    /// * `count` - Optional maximum number of entries to return
    ///
    /// Returns the entries in reverse order, or None if key is not a stream.
    pub fn xrevrange(
        &self,
        key: &str,
        start: &[u8],
        end: &[u8],
        count: Option<usize>,
    ) -> Option<Vec<StreamEntry>> {
        self.xrange(key, start, end, count).map(|mut entries| {
            entries.reverse();
            entries
        })
    }

    fn generate_stream_id(&self) -> Vec<u8> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        format!("{}-0", timestamp).into_bytes()
    }
}

impl Default for StorageEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A trait for converting values into Redis command arguments.
///
/// This trait is implemented for common Rust types to allow them
/// to be used as keys or values in Redis commands.
///
/// # Implementors
///
/// - `String`, `&str`: Converts to Redis string
/// - `Vec<u8>`: Converts to Redis string (raw bytes)
/// - `i64`, `u64`, `isize`, `usize`: Converts to Redis integer
/// - `bool`: Converts to Redis boolean
/// - `Option<T>`: Converts `None` to null, `Some` to the inner value
#[allow(missing_docs)]
pub trait ToRedisArgs {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]>;
}

impl ToRedisArgs for String {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::String(self.as_bytes().to_vec())]
    }
}

impl ToRedisArgs for &str {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::String(self.as_bytes().to_vec())]
    }
}

impl ToRedisArgs for Vec<u8> {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::String(self.clone())]
    }
}

impl ToRedisArgs for i64 {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::Int(*self)]
    }
}

impl ToRedisArgs for u64 {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::Int(*self as i64)]
    }
}

impl ToRedisArgs for isize {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::Int(*self as i64)]
    }
}

impl ToRedisArgs for usize {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::Int(*self as i64)]
    }
}

impl ToRedisArgs for bool {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        smallvec![Value::Bool(*self)]
    }
}

impl<T: ToRedisArgs> ToRedisArgs for Option<T> {
    fn to_redis_args(&self) -> smallvec::SmallVec<[Value; 1]> {
        match self {
            Some(v) => v.to_redis_args(),
            None => smallvec![Value::Null],
        }
    }
}

/// A trait for converting Redis values into Rust types.
///
/// This trait is implemented for common Rust types to allow them
/// to be returned from Redis commands.
///
/// # Implementors
///
/// - `String`: Converts from Redis strings and integers
/// - `Vec<u8>`: Converts from Redis strings (raw bytes)
/// - `i64`: Converts from Redis integers and strings
/// - `bool`: Converts from Redis booleans and integers
/// - `Option<T>`: Converts null to `None`, otherwise `Some(T)`
/// - `Vec<T>`: Converts from Redis arrays
/// - `Value`: Returns the value as-is
#[allow(missing_docs)]
pub trait FromRedisValue: Sized {
    fn from_redis_value(v: Value) -> RedisResult<Self>;
}

impl FromRedisValue for String {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::String(s) => String::from_utf8(s).map_err(|_| RedisError::ParseError),
            Value::Int(n) => Ok(n.to_string()),
            Value::Null => Ok(String::new()),
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
                .ok()
                .and_then(|s| s.parse().ok())
                .ok_or(RedisError::ParseError),
            Value::Bool(b) => Ok(if b { 1 } else { 0 }),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for bool {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        match v {
            Value::Bool(b) => Ok(b),
            Value::Int(n) => Ok(n != 0),
            Value::String(s) => {
                let s_str = String::from_utf8(s).map_err(|_| RedisError::ParseError)?;
                Ok(s_str == "1" || s_str.eq_ignore_ascii_case("true"))
            }
            Value::Null => Ok(false),
            _ => Err(RedisError::ParseError),
        }
    }
}

impl FromRedisValue for Value {
    fn from_redis_value(v: Value) -> RedisResult<Self> {
        Ok(v)
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

/// A Redis client for executing commands against an in-memory store.
///
/// The client provides methods for all common Redis operations including
/// strings, lists, sets, hashes, and sorted sets.
///
/// # Example
///
/// ```rust,no_run
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
        Self {
            storage: StorageEngine::new(),
        }
    }

    /// Creates a new Client with an existing storage engine.
    ///
    /// This allows sharing a storage engine between multiple clients.
    pub fn from_storage(storage: StorageEngine) -> Self {
        Self { storage }
    }

    /// Starts the client, initializing the background expiration sweeper.
    ///
    /// This must be called before using the client to ensure expired keys
    /// are properly cleaned up.
    pub async fn start(&self) {
        self.storage.start_expiration_sweeper().await;
    }

    /// Gets a value from the database.
    ///
    /// # Type Parameters
    /// * `K` - The key type (must be convertible to `String`)
    /// * `RV` - The return value type (must implement [`FromRedisValue`])
    pub async fn get<K: Into<String>, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        RV: FromRedisValue,
    {
        let key_str = key.into();
        if let Some(stored) = self.storage.data.get(&key_str) {
            if stored.is_expired() {
                // Drop the Ref before removing to avoid potential deadlock
                drop(stored);
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Null);
            }
            match &*stored.data {
                RedisData::String(s) => RV::from_redis_value(Value::String(s.clone())),
                _ => Err(RedisError::WrongType),
            }
        } else {
            FromRedisValue::from_redis_value(Value::Null)
        }
    }

    /// Sets a key-value pair in the database.
    ///
    /// # Type Parameters
    /// * `K` - The key type (must be convertible to `String`)
    /// * `V` - The value type
    pub async fn set<K: Into<String>, V>(&mut self, key: K, value: V) -> RedisResult<()>
    where
        V: ToRedisArgs,
    {
        let key_str = key.into();
        let val = Self::value_to_vec(&value);
        self.storage.set(key_str, RedisData::String(val), None);
        Ok(())
    }

    /// Deletes one or more keys from the database.
    ///
    /// Returns the number of keys that were deleted.
    pub async fn del<K>(&mut self, key: K) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        Ok(if self.storage.remove(&key_str) { 1 } else { 0 })
    }

    /// Checks if one or more keys exist in the database.
    ///
    /// Returns `true` if at least one key exists, `false` otherwise.
    pub async fn exists<K>(&mut self, key: K) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        Ok(self.storage.exists(&Self::key_to_string(&key)))
    }

    /// Sets an expiration time on a key.
    ///
    /// # Arguments
    /// * `key` - The key to set expiration on
    /// * `seconds` - The number of seconds until expiration
    ///
    /// Returns `true` if the expiration was set, `false` if the key doesn't exist.
    pub async fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        Ok(self.storage.set_expiry(
            &Self::key_to_string(&key),
            Duration::from_secs(seconds as u64),
        ))
    }

    /// Gets the time-to-live of a key.
    ///
    /// Returns:
    /// - `-1` if the key exists but has no expiration
    /// - `-2` if the key does not exist
    /// - A non-negative value representing seconds until expiration
    pub async fn ttl<K>(&mut self, key: K) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        Ok(self.storage.ttl_query(&Self::key_to_string(&key)))
    }

    /// Sets a field in a hash.
    ///
    /// # Type Parameters
    /// * `K` - The hash key (must be convertible to `String`)
    /// * `F` - The field name
    /// * `V` - The field value
    ///
    /// Returns `1` if the field is new, `0` if the field was updated.
    pub async fn hset<K: Into<String>, F, V>(&mut self, key: K, field: F, value: V) -> RedisResult<i64>
    where
        F: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = key.into();
        let field_b = Self::value_to_vec(&field);
        let value_b = Self::value_to_vec(&value);
        let is_new = if let Some(mut stored) = self.storage.data.get_mut(&key_str) {
            let data_ref = Arc::make_mut(&mut stored.data);
            match data_ref {
                RedisData::Hash(h) => h.insert(field_b, value_b).is_none(),
                _ => return Err(RedisError::WrongType),
            }
        } else {
            // Pre-allocate capacity to reduce rehashing during prepopulation & batch
            let mut h = FxHashMap::default();
            h.reserve(200);
            h.insert(field_b, value_b);
            self.storage.set(key_str, RedisData::Hash(h), None);
            true
        };
        Ok(if is_new { 1 } else { 0 })
    }

    /// Gets a field value from a hash.
    ///
    /// # Type Parameters
    /// * `K` - The hash key (must be convertible to `String`)
    /// * `F` - The field name
    /// * `RV` - The return value type
    pub async fn hget<K: Into<String>, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        F: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = key.into();
        let field_b = Self::value_to_vec(&field);
        if let Some(stored) = self.storage.data.get(&key_str) {
            if stored.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Null);
            }
            match &*stored.data {
                RedisData::Hash(h) => {
                    if let Some(v) = h.get(&field_b) {
                        RV::from_redis_value(Value::String(v.clone()))
                    } else {
                        FromRedisValue::from_redis_value(Value::Null)
                    }
                }
                _ => Err(RedisError::WrongType),
            }
        } else {
            FromRedisValue::from_redis_value(Value::Null)
        }
    }

    /// Gets all fields and values from a hash.
    ///
    /// Returns an array of alternating field names and values.
    pub async fn hgetall<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = Self::key_to_string(&key);
        if let Some(stored) = self.storage.data.get(&key_str) {
            if stored.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Array(Vec::new()));
            }
            match &*stored.data {
                RedisData::Hash(h) => {
                    let mut res = Vec::with_capacity(h.len() * 2);
                    for (k, v) in h.iter() {
                        res.push(Value::String(k.clone()));
                        res.push(Value::String(v.clone()));
                    }
                    FromRedisValue::from_redis_value(Value::Array(res))
                }
                _ => Err(RedisError::WrongType),
            }
        } else {
            FromRedisValue::from_redis_value(Value::Array(Vec::new()))
        }
    }

    /// Deletes one or more fields from a hash.
    ///
    /// Returns the number of fields that were deleted.
    pub async fn hdel<K, F>(&mut self, key: K, field: F) -> RedisResult<i64>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let field_b = Self::value_to_vec(&field);
        if let Some(mut stored) = self.storage.data.get_mut(&key_str) {
            let data_ref = Arc::make_mut(&mut stored.data);
            match data_ref {
                RedisData::Hash(h) => {
                    let existed = h.remove(&field_b).is_some();
                    return Ok(if existed { 1 } else { 0 });
                }
                _ => return Err(RedisError::WrongType),
            }
        }
        Ok(0)
    }

    /// Pushes a value to the front (left) of a list.
    ///
    /// Returns the length of the list after the push.
    pub async fn lpush<K: Into<String>, V>(&mut self, key: K, value: V) -> RedisResult<i64>
    where
        V: ToRedisArgs,
    {
        let key_str = key.into();
        let val_b = Self::value_to_vec(&value);
        let len = if let Some(mut stored) = self.storage.data.get_mut(&key_str) {
            let data_ref = Arc::make_mut(&mut stored.data);
            match data_ref {
                RedisData::List(l) => {
                    l.push_front(val_b);
                    l.len() as i64
                }
                _ => return Err(RedisError::WrongType),
            }
        } else {
            let mut l = VecDeque::new();
            l.push_front(val_b);
            self.storage.set(key_str, RedisData::List(l), None);
            1
        };
        Ok(len)
    }

    /// Pushes a value to the back (right) of a list.
    ///
    /// Returns the length of the list after the push.
    pub async fn rpush<K: Into<String>, V>(&mut self, key: K, value: V) -> RedisResult<i64>
    where
        V: ToRedisArgs,
    {
        let key_str = key.into();
        let val_b = Self::value_to_vec(&value);
        let len = if let Some(mut stored) = self.storage.data.get_mut(&key_str) {
            let data_ref = Arc::make_mut(&mut stored.data);
            match data_ref {
                RedisData::List(l) => {
                    l.push_back(val_b);
                    l.len() as i64
                }
                _ => return Err(RedisError::WrongType),
            }
        } else {
            let mut l = VecDeque::new();
            l.push_back(val_b);
            self.storage.set(key_str, RedisData::List(l), None);
            1
        };
        Ok(len)
    }

    /// Returns the length of a list.
    ///
    /// Returns `0` if the key doesn't exist or is not a list.
    pub async fn llen<K>(&mut self, key: K) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        if let Some(stored) = self.storage.data.get(&key_str) {
            if stored.is_expired() {
                drop(stored);
                self.storage.remove(&key_str);
                return Ok(0);
            }
            match &*stored.data {
                RedisData::List(l) => Ok(l.len() as i64),
                _ => Err(RedisError::WrongType),
            }
        } else {
            Ok(0)
        }
    }

    /// Adds one or more members to a set.
    ///
    /// Returns the number of members that were added to the set.
    pub async fn sadd<K: Into<String>, V>(&mut self, key: K, member: V) -> RedisResult<i64>
    where
        V: ToRedisArgs,
    {
        let key_str = key.into();
        let member_b = Self::value_to_vec(&member);
        if let Some(mut stored) = self.storage.data.get_mut(&key_str) {
            let data_ref = Arc::make_mut(&mut stored.data);
            match data_ref {
                RedisData::Set(s) => {
                    let added = s.insert(member_b);
                    Ok(if added { 1 } else { 0 })
                }
                _ => return Err(RedisError::WrongType),
            }
        } else {
            let mut s = FxHashSet::default();
            s.insert(member_b);
            self.storage.set(key_str, RedisData::Set(s), None);
            Ok(1)
        }
    }

    /// Returns all members of a set.
    pub async fn smembers<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = Self::key_to_string(&key);
        if let Some(stored) = self.storage.data.get(&key_str) {
            if stored.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Array(Vec::new()));
            }
            match &*stored.data {
                RedisData::Set(s) => {
                    let members: Vec<Value> = s.iter().map(|m| Value::String(m.clone())).collect();
                    FromRedisValue::from_redis_value(Value::Array(members))
                }
                _ => Err(RedisError::WrongType),
            }
        } else {
            FromRedisValue::from_redis_value(Value::Array(Vec::new()))
        }
    }

    /// Pings the server.
    ///
    /// Returns "PONG".
    pub async fn ping(&mut self) -> RedisResult<String> {
        Ok("PONG".to_string())
    }

    /// Echoes the given message.
    ///
    /// Returns the message that was passed in.
    pub async fn echo<K>(&mut self, msg: K) -> RedisResult<String>
    where
        K: ToRedisArgs,
    {
        Ok(String::from_utf8_lossy(&Self::value_to_vec(&msg)).to_string())
    }

    /// Returns the number of keys in the database.
    pub async fn dbsize(&mut self) -> RedisResult<i64> {
        Ok(self.storage.len() as i64)
    }

    /// Removes all keys from the current database.
    ///
    /// Returns "OK".
    pub async fn flushdb(&mut self) -> RedisResult<String> {
        self.storage.flush();
        Ok("OK".to_string())
    }

    /// Removes the expiration from a key.
    ///
    /// Returns `true` if the key existed and expiration was removed, `false` otherwise.
    pub async fn persist(&mut self, key: &str) -> bool {
        self.storage.persist(key)
    }

    /// Adds an entry to a stream.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `entry_id` - Optional custom entry ID (format: "timestamp-sequence"). If None, auto-generates
    /// * `values` - Field-value pairs to add
    ///
    /// Returns the entry ID as a string.
    pub async fn xadd<K, F, V>(
        &mut self,
        key: K,
        entry_id: Option<&str>,
        values: Vec<(F, V)>,
    ) -> RedisResult<String>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let entry_id_bytes = entry_id.map(|s| s.as_bytes().to_vec());
        let values: Vec<(Vec<u8>, Vec<u8>)> = values
            .into_iter()
            .map(|(f, v)| (Self::value_to_vec(&f), Self::value_to_vec(&v)))
            .collect();

        match self
            .storage
            .xadd(&key_str, entry_id_bytes.as_deref(), values)
        {
            Some(id) => Ok(String::from_utf8_lossy(&id).to_string()),
            None => Err(RedisError::WrongType),
        }
    }

    /// Returns the number of entries in a stream.
    pub async fn xlen<K>(&mut self, key: K) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        Ok(self.storage.xlen(&key_str).unwrap_or(0) as i64)
    }

    /// Trims a stream to a maximum number of entries.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `maxlen` - Maximum number of entries to keep
    /// * `approximate` - If true, uses approximate trimming (trim to maxlen - 10%)
    ///
    /// Returns the number of entries removed.
    pub async fn xtrim<K>(&mut self, key: K, maxlen: usize, approximate: bool) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        Ok(self
            .storage
            .xtrim(&key_str, maxlen, approximate)
            .unwrap_or(0) as i64)
    }

    /// Deletes entries from a stream.
    ///
    /// Returns the number of entries deleted.
    pub async fn xdel<K>(&mut self, key: K, entry_ids: Vec<&str>) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let ids: Vec<&[u8]> = entry_ids.iter().map(|s| s.as_bytes()).collect();
        Ok(self.storage.xdel(&key_str, ids).unwrap_or(0) as i64)
    }

    /// Returns entries in a stream within a range.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `start` - Start ID ("-" for beginning)
    /// * `end` - End ID ("+" for end)
    /// * `count` - Optional maximum number of entries to return
    pub async fn xrange<K, RV>(
        &mut self,
        key: K,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = Self::key_to_string(&key);
        let entries = self
            .storage
            .xrange(&key_str, start.as_bytes(), end.as_bytes(), count);

        match entries {
            Some(entries) => {
                let values: Vec<Value> = entries
                    .into_iter()
                    .map(|(id, fields)| {
                        let mut arr = vec![Value::String(id)];
                        for (field, value) in fields {
                            arr.push(Value::String(field));
                            arr.push(Value::String(value));
                        }
                        Value::Array(arr)
                    })
                    .collect();
                FromRedisValue::from_redis_value(Value::Array(values))
            }
            None => FromRedisValue::from_redis_value(Value::Array(Vec::new())),
        }
    }

    /// Returns entries in a stream within a range, in reverse order.
    ///
    /// # Arguments
    /// * `key` - The stream key
    /// * `start` - Start ID ("+" for end)
    /// * `end` - End ID ("-" for beginning)
    /// * `count` - Optional maximum number of entries to return
    pub async fn xrevrange<K, RV>(
        &mut self,
        key: K,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = Self::key_to_string(&key);
        let entries = self
            .storage
            .xrevrange(&key_str, start.as_bytes(), end.as_bytes(), count);

        match entries {
            Some(entries) => {
                let values: Vec<Value> = entries
                    .into_iter()
                    .map(|(id, fields)| {
                        let mut arr = vec![Value::String(id)];
                        for (field, value) in fields {
                            arr.push(Value::String(field));
                            arr.push(Value::String(value));
                        }
                        Value::Array(arr)
                    })
                    .collect();
                FromRedisValue::from_redis_value(Value::Array(values))
            }
            None => FromRedisValue::from_redis_value(Value::Array(Vec::new())),
        }
    }

    fn key_to_string<K: ToRedisArgs>(key: &K) -> String {
        let bytes = Self::value_to_vec(key);
        // If the bytes are valid UTF-8, String::from_utf8 will take ownership of the Vec without copying.
        // If not valid (unlikely for keys), fall back to lossy conversion.
        String::from_utf8(bytes).unwrap_or_else(|err| {
            let bytes = err.into_bytes();
            String::from_utf8_lossy(&bytes).to_string()
        })
    }

    fn value_to_vec<V: ToRedisArgs>(v: &V) -> Vec<u8> {
        let args = v.to_redis_args();
        for arg in args {
            match arg {
                Value::String(s) => return s,
                Value::Int(n) => return n.to_string().into_bytes(),
                Value::Bool(b) => return (if b { "1" } else { "0" }).to_string().into_bytes(),
                _ => {}
            }
        }
        Vec::new()
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xtrim_shrinks_vec_capacity() {
        let engine = StorageEngine::new();

        // Add many entries to build up Vec capacity
        for i in 0..100 {
            let id = format!("{}-0", i);
            engine.xadd(
                "stream",
                Some(id.as_bytes()),
                vec![(b"k".to_vec(), b"v".to_vec())],
            );
        }

        // Verify we have 100 entries
        assert_eq!(engine.xlen("stream"), Some(100));

        // Get capacity before trim
        let capacity_before = match &*engine.data.get("stream").unwrap().data {
            RedisData::Stream(entries) => entries.capacity(),
            _ => panic!("expected stream"),
        };

        // Trim to keep only 2 entries
        let removed = engine.xtrim("stream", 2, false);
        assert_eq!(removed, Some(98));
        assert_eq!(engine.xlen("stream"), Some(2));

        // Verify capacity shrunk
        let capacity_after = match &*engine.data.get("stream").unwrap().data {
            RedisData::Stream(entries) => entries.capacity(),
            _ => panic!("expected stream"),
        };

        assert!(
            capacity_after < capacity_before,
            "capacity should shrink after trim: before={}, after={}",
            capacity_before,
            capacity_after
        );
    }

    #[test]
    fn test_xdel_shrinks_vec_capacity() {
        let engine = StorageEngine::new();

        // Add many entries
        let mut ids = Vec::new();
        for i in 0..100 {
            let id = format!("{}-0", i);
            engine.xadd(
                "stream",
                Some(id.as_bytes()),
                vec![(b"k".to_vec(), b"v".to_vec())],
            );
            ids.push(id);
        }

        // Get capacity before delete
        let capacity_before = match &*engine.data.get("stream").unwrap().data {
            RedisData::Stream(entries) => entries.capacity(),
            _ => panic!("expected stream"),
        };

        // Delete most entries (keep only the last 2)
        let to_delete: Vec<&[u8]> = ids[..98].iter().map(|s| s.as_bytes()).collect();
        let removed = engine.xdel("stream", to_delete);
        assert_eq!(removed, Some(98));

        // Verify capacity shrunk
        let capacity_after = match &*engine.data.get("stream").unwrap().data {
            RedisData::Stream(entries) => entries.capacity(),
            _ => panic!("expected stream"),
        };

        assert!(
            capacity_after < capacity_before,
            "capacity should shrink after xdel: before={}, after={}",
            capacity_before,
            capacity_after
        );
    }

    #[test]
    fn test_compact_preserves_data() {
        let engine = StorageEngine::new();
        engine.set("key1", RedisData::String(b"val1".to_vec()), None);
        engine.set("key2", RedisData::String(b"val2".to_vec()), None);

        engine.compact();

        assert_eq!(engine.len(), 2);
        assert!(engine.exists("key1"));
        assert!(engine.exists("key2"));
    }

    #[test]
    fn test_compact_resets_high_water_mark() {
        let engine = StorageEngine::new();
        for i in 0..100 {
            engine.set(
                &format!("key{}", i),
                RedisData::String(b"val".to_vec()),
                None,
            );
        }
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 100);

        // Remove some keys without triggering auto-compact (50 >= 25% of 100)
        for i in 50..100 {
            engine.remove(&format!("key{}", i));
        }
        assert_eq!(engine.len(), 50);

        // Manual compact should reset high-water mark
        engine.compact();
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 50);
    }

    #[test]
    fn test_auto_compaction_on_remove() {
        let engine = StorageEngine::new();
        for i in 0..100 {
            engine.set(
                &format!("key{}", i),
                RedisData::String(b"val".to_vec()),
                None,
            );
        }
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 100);

        // Remove keys until len < 25% of high-water mark (below 25)
        for i in 0..76 {
            engine.remove(&format!("key{}", i));
        }

        // After auto-compaction triggered, high-water mark should be reset
        assert_eq!(engine.len(), 24);
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 24);
    }

    #[test]
    fn test_no_auto_compaction_above_threshold() {
        let engine = StorageEngine::new();
        for i in 0..100 {
            engine.set(
                &format!("key{}", i),
                RedisData::String(b"val".to_vec()),
                None,
            );
        }

        // Remove only 50 keys — 50 remaining is >= 25% of 100
        for i in 0..50 {
            engine.remove(&format!("key{}", i));
        }

        // High-water mark should NOT have been reset
        assert_eq!(engine.len(), 50);
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_flush_resets_high_water_mark() {
        let engine = StorageEngine::new();
        for i in 0..50 {
            engine.set(
                &format!("key{}", i),
                RedisData::String(b"val".to_vec()),
                None,
            );
        }
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 50);

        engine.flush();
        assert_eq!(engine.high_water_mark.load(Ordering::Relaxed), 0);
        assert_eq!(engine.len(), 0);
    }
}
