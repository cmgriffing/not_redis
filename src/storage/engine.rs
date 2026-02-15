//! The core storage engine implementation.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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
    pub fn new() -> Self {
        Self::new_with_sweep_interval(100)
    }

    pub fn new_with_sweep_interval(sweep_interval_ms: u64) -> Self {
        let engine = Self {
            data: Arc::new(DashMap::new()),
            expiration: ExpirationManager::new(sweep_interval_ms),
        };
        engine
    }

    pub fn new(sweep_interval_ms: u64) -> Self {
        Self::new_with_sweep_interval(sweep_interval_ms)
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

    pub fn xadd(&self, key: &str, entry_id: Option<&[u8]>, values: Vec<(Vec<u8>, Vec<u8>)>) -> Option<Vec<u8>> {
        let new_id = match entry_id {
            Some(id) => id.to_vec(),
            None => self.generate_stream_id(),
        };

        let entry = (new_id.clone(), values);

        if let Some(mut stored) = self.data.get_mut(key) {
            match &mut stored.data {
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

    pub fn xlen(&self, key: &str) -> Option<usize> {
        self.data.get(key).map(|stored| {
            match &stored.data {
                RedisData::Stream(entries) => entries.len(),
                _ => 0,
            }
        })
    }

    pub fn xtrim(&self, key: &str, maxlen: usize, approximate: bool) -> Option<usize> {
        if let Some(mut stored) = self.data.get_mut(key) {
            match &mut stored.data {
                RedisData::Stream(entries) => {
                    let original_len = entries.len();
                    if entries.len() > maxlen {
                        let keep = if approximate {
                            maxlen.saturating_sub(maxlen / 10)
                        } else {
                            maxlen
                        };
                        entries.drain(0..entries.len().saturating_sub(keep));
                    }
                    return Some(original_len.saturating_sub(entries.len()));
                }
                _ => return None,
            }
        }
        None
    }

    pub fn xdel(&self, key: &str, entry_ids: Vec<&[u8]>) -> Option<usize> {
        if let Some(mut stored) = self.data.get_mut(key) {
            match &mut stored.data {
                RedisData::Stream(entries) => {
                    let original_len = entries.len();
                    entries.retain(|(id, _)| {
                        !entry_ids.iter().any(|del_id| *del_id == id.as_slice())
                    });
                    return Some(original_len.saturating_sub(entries.len()));
                }
                _ => return None,
            }
        }
        None
    }

    pub fn xrange(&self, key: &str, start: &[u8], end: &[u8], count: Option<usize>) -> Option<Vec<(Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>)>> {
        self.data.get(key).map(|stored| {
            match &stored.data {
                RedisData::Stream(entries) => {
                    let mut result: Vec<_> = entries.iter()
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
            }
        })
    }

    pub fn xrevrange(&self, key: &str, start: &[u8], end: &[u8], count: Option<usize>) -> Option<Vec<(Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>)>> {
        self.xrange(key, start, end, count).map(|mut entries| {
            entries.reverse();
            entries
        })
    }

    fn generate_stream_id(&self) -> Vec<u8> {
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
