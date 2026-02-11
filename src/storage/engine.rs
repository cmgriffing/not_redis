//! The core storage engine implementation.

use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;

use super::types::{RedisData, StoredValue};
use super::expire::ExpirationManager;

/// The core storage engine for the Redis-like store.
/// 
/// Uses a concurrent hash map ([`DashMap`]) for thread-safe access
/// and supports key expiration with a background sweeper task.
#[derive(Clone)]
pub struct StorageEngine {
    data: Arc<DashMap<String, StoredValue>>,
    expiration: ExpirationManager,
}

impl StorageEngine {
    pub fn new(sweep_interval_ms: u64) -> Self {
        let engine = Self {
            data: Arc::new(DashMap::new()),
            expiration: ExpirationManager::new(sweep_interval_ms),
        };
        engine
    }

    pub async fn start_expiration_sweeper(&self) {
        let expiration = self.expiration.clone();
        let data = Arc::clone(&self.data);
        
        tokio::spawn(async move {
            expiration.start_sweep_task(move |key| {
                data.remove(key);
            }).await;
        });
    }

    pub fn set(&self, key: &str, value: RedisData, expire_at: Option<Instant>) {
        if let Some(old) = self.data.get(key) {
            if old.expire_at.is_some() {
                self.expiration.cancel_expiration(key);
            }
        }

        let stored = StoredValue {
            data: value,
            expire_at,
        };

        self.data.insert(key.to_string(), stored);

        if let Some(at) = expire_at {
            self.expiration.schedule_expiration(key.to_string(), at);
        }
    }

    pub fn get(&self, key: &str) -> Option<StoredValue> {
        self.data.get(key).map(|v| v.clone())
    }

    pub fn remove(&self, key: &str) -> bool {
        self.expiration.cancel_expiration(key);
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

    pub fn keys(&self) -> Vec<String> {
        self.data.iter().map(|kv| kv.key().clone()).collect()
    }

    pub fn flush(&self) {
        self.data.clear();
        self.expiration.clear_all();
    }

    pub fn set_expiry(&self, key: &str, duration: Duration) -> bool {
        if let Some(mut entry) = self.data.get_mut(key) {
            let expire_at = Instant::now() + duration;
            entry.expire_at = Some(expire_at);
            self.expiration.schedule_expiration(key.to_string(), expire_at);
            return true;
        }
        false
    }

    pub fn persist(&self, key: &str) -> bool {
        if let Some(mut entry) = self.data.get_mut(key) {
            entry.expire_at = None;
            self.expiration.cancel_expiration(key);
            return true;
        }
        false
    }

    pub fn ttl(&self, key: &str) -> Option<Duration> {
        self.data.get(key).and_then(|entry| {
            entry.expire_at.map(|at| {
                at.saturating_duration_since(Instant::now())
            })
        })
    }
}
