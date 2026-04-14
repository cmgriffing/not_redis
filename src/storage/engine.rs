//! The core storage engine implementation.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use dashmap::DashMap;

use super::config::{MaxMemoryPolicy, StorageConfig};
use super::memory::MemoryTracker;
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
    high_water_mark: Arc<AtomicUsize>,
    current_len: Arc<AtomicUsize>,
}

#[allow(missing_docs)]
impl StorageEngine {
    pub fn new() -> Self {
        Self::new_with_sweep_interval(100)
    }

    pub fn new_with_sweep_interval(sweep_interval_ms: u64) -> Self {
        let engine = Self {
            data: Arc::new(DashMap::with_hasher_and_shard_amount(
                rustc_hash::FxBuildHasher::default(),
                1,
            )),
            expiration: ExpirationManager::new(sweep_interval_ms),
            high_water_mark: Arc::new(AtomicUsize::new(0)),
            current_len: Arc::new(AtomicUsize::new(0)),
        };
        engine
    }

    pub fn new() -> Self {
        Self::new_with_sweep_interval(100)
    }

    pub fn with_config(config: StorageConfig) -> Self {
        let mut engine = Self {
            data: Arc::new(DashMap::with_hasher_and_shard_amount(
                rustc_hash::FxBuildHasher::default(),
                1,
            )),
            expiration: ExpirationManager::new(100),
            memory: MemoryTracker::new(),
        };
        
        if let Some(maxmemory) = config.maxmemory {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(engine.memory.set_maxmemory(Some(maxmemory)));
        }
        
        let rt = tokio::runtime::Handle::current();
        rt.block_on(engine.memory.set_maxmemory_policy(config.maxmemory_policy));
        
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
        let memory_enabled = self.memory.is_enabled_sync();
        
        if !memory_enabled {
            self.data.insert(
                key.to_string(),
                StoredValue { data: value, expire_at },
            );
            return;
        }

        if self.memory.should_reject_write_sync() {
            return;
        }

        match self.data.entry(key.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                if entry.get().expire_at.is_some() {
                    self.expiration.cancel_expiration(entry.key());
                }
                entry.insert(StoredValue { data: value, expire_at });
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(StoredValue { data: value, expire_at });
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<StoredValue> {
        let value = self.data.get(key).map(|v| v.clone());
        
        if value.is_some() && self.memory.is_enabled_sync() {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.memory.record_read(key));
        }
        
        value
    }

    pub fn remove(&self, key: &str) -> bool {
        let memory_enabled = self.memory.is_enabled_sync();
        if !memory_enabled {
            return self.data.remove(key).is_some();
        }
        if let Some(old) = self.data.get(key) {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.memory.remove_memory(key, &old));
        }
        
        self.expiration.cancel_expiration(key);
        self.data.remove(key).is_some()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn set_no_replace(&self, key: &str, value: RedisData, expire_at: Option<Instant>) {
        let memory_enabled = self.memory.is_enabled_sync();

        if !memory_enabled {
            self.data.insert(
                key.to_string(),
                StoredValue { data: value, expire_at },
            );
            return;
        }

        if self.memory.should_reject_write_sync() {
            return;
        }

        match self.data.entry(key.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                if entry.get().expire_at.is_some() {
                    self.expiration.cancel_expiration(entry.key());
                }
                entry.insert(StoredValue { data: value, expire_at });
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(StoredValue { data: value, expire_at });
            }
        }
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
        self.expiration.clear();
    }

    pub fn set_expiry(&self, key: &str, duration: Duration) -> bool {
        if let Some(mut entry) = self.data.get_mut(key) {
            let at = Instant::now() + duration;
            entry.expire_at = Some(at);
            self.expiration.schedule(key.to_string(), at);
            return true;
        }
        false
    }

    pub fn persist(&self, key: &str) -> bool {
        if let Some(mut entry) = self.data.get_mut(key) {
            entry.expire_at = None;
            self.expiration.cancel(key);
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

    pub fn xlen(&self, key: &str) -> Option<usize> {
        self.data.get(key).map(|stored| {
            match &*stored.data {
                RedisData::Stream(entries) => entries.len(),
                _ => 0,
            }
        })
    }

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
            match Arc::make_mut(&mut stored.data) {
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
            match &*stored.data {
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