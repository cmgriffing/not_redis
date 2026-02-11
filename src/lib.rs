use dashmap::DashMap;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
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

pub type RedisResult<T> = Result<T, RedisError>;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub enum RedisData {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    Hash(HashMap<Vec<u8>, Vec<u8>>),
    ZSet(BTreeMap<Vec<u8>, f64>),
}

#[derive(Debug, Clone)]
pub struct StoredValue {
    pub data: RedisData,
    pub expire_at: Option<Instant>,
}

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

#[derive(Clone)]
pub struct StorageEngine {
    data: Arc<DashMap<String, StoredValue>>,
    expiration: ExpirationManager,
}

impl StorageEngine {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            expiration: ExpirationManager::new(100),
        }
    }

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

    pub fn set(&self, key: &str, value: RedisData, expire_at: Option<Instant>) {
        if let Some(old) = self.data.get(key)
            && old.expire_at.is_some()
        {
            self.expiration.cancel(key);
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

    pub fn get(&self, key: &str) -> Option<StoredValue> {
        self.data.get(key).map(|v| v.clone())
    }

    pub fn remove(&self, key: &str) -> bool {
        self.expiration.cancel(key);
        self.data.remove(key).is_some()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn flush(&self) {
        self.data.clear();
        self.expiration.clear();
    }

    pub fn set_expiry(&self, key: &str, dur: Duration) -> bool {
        if let Some(mut e) = self.data.get_mut(key) {
            let at = Instant::now() + dur;
            e.expire_at = Some(at);
            self.expiration.schedule(key.to_string(), at);
            return true;
        }
        false
    }

    pub fn persist(&self, key: &str) -> bool {
        if let Some(mut e) = self.data.get_mut(key) {
            e.expire_at = None;
            self.expiration.cancel(key);
            return true;
        }
        false
    }

    pub fn ttl(&self, key: &str) -> Option<Duration> {
        self.data.get(key).and_then(|e| {
            e.expire_at
                .map(|at| at.saturating_duration_since(Instant::now()))
        })
    }

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

pub struct Client {
    storage: StorageEngine,
}

impl Client {
    pub fn new() -> Self {
        Self {
            storage: StorageEngine::new(),
        }
    }

    pub async fn start(&self) {
        self.storage.start_expiration_sweeper().await;
    }

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

    pub async fn del<K>(&mut self, key: K) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        let key_str = Self::key_to_string(&key);
        Ok(if self.storage.remove(&key_str) { 1 } else { 0 })
    }

    pub async fn exists<K>(&mut self, key: K) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        Ok(self.storage.exists(&Self::key_to_string(&key)))
    }

    pub async fn expire<K>(&mut self, key: K, seconds: i64) -> RedisResult<bool>
    where
        K: ToRedisArgs,
    {
        Ok(self.storage.set_expiry(
            &Self::key_to_string(&key),
            Duration::from_secs(seconds as u64),
        ))
    }

    pub async fn ttl<K>(&mut self, key: K) -> RedisResult<i64>
    where
        K: ToRedisArgs,
    {
        Ok(self.storage.ttl_query(&Self::key_to_string(&key)))
    }

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

    pub async fn ping(&mut self) -> RedisResult<String> {
        Ok("PONG".to_string())
    }

    pub async fn echo<K>(&mut self, msg: K) -> RedisResult<String>
    where
        K: ToRedisArgs,
    {
        Ok(String::from_utf8_lossy(&Self::value_to_vec(&msg)).to_string())
    }

    pub async fn dbsize(&mut self) -> RedisResult<i64> {
        Ok(self.storage.len() as i64)
    }

    pub async fn flushdb(&mut self) -> RedisResult<String> {
        self.storage.flush();
        Ok("OK".to_string())
    }

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
