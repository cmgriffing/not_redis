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

use dashmap::DashMap;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
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

/// Internal data types stored in the engine.
///
/// These represent the actual data structures that can be stored,
/// as opposed to the RESP protocol [`Value`] types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum RedisData {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    Hash(HashMap<Vec<u8>, Vec<u8>>),
    ZSet(BTreeMap<Vec<u8>, f64>),
}

/// A value stored in the storage engine with optional expiration.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct StoredValue {
    pub data: RedisData,
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
    expirations: Arc<Mutex<BTreeMap<Instant, HashSet<String>>>>,
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

/// The core storage engine for the Redis-like store.
///
/// Uses a concurrent hash map ([`DashMap`]) for thread-safe access
/// and supports key expiration with a background sweeper task.
#[derive(Clone)]
pub struct StorageEngine {
    data: Arc<DashMap<String, StoredValue>>,
    expiration: ExpirationManager,
}

#[allow(missing_docs)]
impl StorageEngine {
    /// Creates a new storage engine with the default sweep interval.
    ///
    /// The sweep interval determines how often expired keys are cleaned up
    /// by the background task when started.
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            expiration: ExpirationManager::new(100),
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
    pub fn set(&self, key: &str, value: RedisData, expire_at: Option<Instant>) {
        if let Some(old) = self.data.get(key) {
            if old.expire_at.is_some() {
                self.expiration.cancel(key);
            }
        }
        let stored = StoredValue {
            data: value,
            expire_at,
        };
        self.data.insert(key.to_string(), stored);
        if let Some(at) = expire_at {
            self.expiration.schedule(key.to_string(), at);
        }
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
        self.data.remove(key).is_some()
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
    fn to_redis_args(&self) -> Vec<Value>;
}

impl ToRedisArgs for String {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::String(self.as_bytes().to_vec())]
    }
}

impl ToRedisArgs for &str {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::String(self.as_bytes().to_vec())]
    }
}

impl ToRedisArgs for Vec<u8> {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::String(self.clone())]
    }
}

impl ToRedisArgs for i64 {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::Int(*self)]
    }
}

impl ToRedisArgs for u64 {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::Int(*self as i64)]
    }
}

impl ToRedisArgs for isize {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::Int(*self as i64)]
    }
}

impl ToRedisArgs for usize {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::Int(*self as i64)]
    }
}

impl ToRedisArgs for bool {
    fn to_redis_args(&self) -> Vec<Value> {
        vec![Value::Bool(*self)]
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
    /// * `K` - The key type (must implement [`ToRedisArgs`])
    /// * `RV` - The return value type (must implement [`FromRedisValue`])
    pub async fn get<K, RV>(&mut self, key: K) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = Self::key_to_string(&key);
        if let Some(val) = self.storage.get(&key_str) {
            if val.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Null);
            }
            match val.data {
                RedisData::String(s) => RV::from_redis_value(Value::String(s)),
                _ => Err(RedisError::WrongType),
            }
        } else {
            FromRedisValue::from_redis_value(Value::Null)
        }
    }

    /// Sets a key-value pair in the database.
    ///
    /// # Type Parameters
    /// * `K` - The key type
    /// * `V` - The value type
    pub async fn set<K, V>(&mut self, key: K, value: V) -> RedisResult<()>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let val = Self::value_to_vec(&value);
        self.storage.set(&key_str, RedisData::String(val), None);
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
    /// * `K` - The hash key
    /// * `F` - The field name
    /// * `V` - The field value
    ///
    /// Returns `1` if the field is new, `0` if the field was updated.
    pub async fn hset<K, F, V>(&mut self, key: K, field: F, value: V) -> RedisResult<i64>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let field_b = Self::value_to_vec(&field);
        let value_b = Self::value_to_vec(&value);
        let is_new = if let Some(val) = self.storage.get(&key_str) {
            match val.data {
                RedisData::Hash(mut h) => {
                    let is_new = !h.contains_key(&field_b);
                    h.insert(field_b.clone(), value_b);
                    self.storage
                        .set(&key_str, RedisData::Hash(h), val.expire_at);
                    is_new
                }
                _ => return Err(RedisError::WrongType),
            }
        } else {
            let mut h = HashMap::new();
            h.insert(field_b.clone(), value_b);
            self.storage.set(&key_str, RedisData::Hash(h), None);
            true
        };
        Ok(if is_new { 1 } else { 0 })
    }

    /// Gets a field value from a hash.
    ///
    /// # Type Parameters
    /// * `K` - The hash key
    /// * `F` - The field name
    /// * `RV` - The return value type
    pub async fn hget<K, F, RV>(&mut self, key: K, field: F) -> RedisResult<RV>
    where
        K: ToRedisArgs,
        F: ToRedisArgs,
        RV: FromRedisValue,
    {
        let key_str = Self::key_to_string(&key);
        let field_b = Self::value_to_vec(&field);
        if let Some(val) = self.storage.get(&key_str) {
            if val.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Null);
            }
            match val.data {
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
        if let Some(val) = self.storage.get(&key_str) {
            if val.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Array(Vec::new()));
            }
            match val.data {
                RedisData::Hash(h) => {
                    let mut res = Vec::new();
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
        if let Some(val) = self.storage.get(&key_str) {
            match val.data {
                RedisData::Hash(mut h) => {
                    let existed = h.remove(&field_b).is_some();
                    if existed {
                        self.storage
                            .set(&key_str, RedisData::Hash(h), val.expire_at);
                    }
                    Ok(if existed { 1 } else { 0 })
                }
                _ => Err(RedisError::WrongType),
            }
        } else {
            Ok(0)
        }
    }

    /// Pushes a value to the front (left) of a list.
    ///
    /// Returns the length of the list after the push.
    pub async fn lpush<K, V>(&mut self, key: K, value: V) -> RedisResult<i64>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let val_b = Self::value_to_vec(&value);
        let len = if let Some(val) = self.storage.get(&key_str) {
            match val.data {
                RedisData::List(mut l) => {
                    l.push_front(val_b);
                    self.storage
                        .set(&key_str, RedisData::List(l.clone()), val.expire_at);
                    l.len() as i64
                }
                _ => return Err(RedisError::WrongType),
            }
        } else {
            let mut l = VecDeque::new();
            l.push_front(val_b);
            self.storage.set(&key_str, RedisData::List(l), None);
            1
        };
        Ok(len)
    }

    /// Pushes a value to the back (right) of a list.
    ///
    /// Returns the length of the list after the push.
    pub async fn rpush<K, V>(&mut self, key: K, value: V) -> RedisResult<i64>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let val_b = Self::value_to_vec(&value);
        let len = if let Some(val) = self.storage.get(&key_str) {
            match val.data {
                RedisData::List(mut l) => {
                    l.push_back(val_b);
                    self.storage
                        .set(&key_str, RedisData::List(l.clone()), val.expire_at);
                    l.len() as i64
                }
                _ => return Err(RedisError::WrongType),
            }
        } else {
            let mut l = VecDeque::new();
            l.push_back(val_b);
            self.storage.set(&key_str, RedisData::List(l), None);
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
        if let Some(val) = self.storage.get(&key_str) {
            if val.is_expired() {
                self.storage.remove(&key_str);
                return Ok(0);
            }
            match val.data {
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
    pub async fn sadd<K, V>(&mut self, key: K, member: V) -> RedisResult<i64>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        let member_b = Self::value_to_vec(&member);
        if let Some(val) = self.storage.get(&key_str) {
            match val.data {
                RedisData::Set(mut s) => {
                    let added = s.insert(member_b.clone());
                    self.storage.set(&key_str, RedisData::Set(s), val.expire_at);
                    Ok(if added { 1 } else { 0 })
                }
                _ => Err(RedisError::WrongType),
            }
        } else {
            let mut s = HashSet::new();
            s.insert(member_b);
            self.storage.set(&key_str, RedisData::Set(s), None);
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
        if let Some(val) = self.storage.get(&key_str) {
            if val.is_expired() {
                self.storage.remove(&key_str);
                return FromRedisValue::from_redis_value(Value::Array(Vec::new()));
            }
            match val.data {
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

    fn key_to_string<K: ToRedisArgs>(key: &K) -> String {
        String::from_utf8_lossy(&Self::value_to_vec(key)).to_string()
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
