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
    memory: MemoryTracker,
}

impl StorageEngine {
    pub fn new() -> Self {
        Self::new_with_sweep_interval(100)
    }

    pub fn new_with_sweep_interval(sweep_interval_ms: u64) -> Self {
        let engine = Self {
            data: Arc::new(DashMap::new()),
            expiration: ExpirationManager::new(sweep_interval_ms),
            memory: MemoryTracker::new(),
        };
        engine
    }

    pub fn new(sweep_interval_ms: u64) -> Self {
        Self::new_with_sweep_interval(sweep_interval_ms)
    }

    pub fn with_config(config: StorageConfig) -> Self {
        let mut engine = Self {
            data: Arc::new(DashMap::new()),
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
        let rt = tokio::runtime::Handle::current();
        
        let memory_enabled = rt.block_on(self.memory.is_enabled());
        
        if memory_enabled && rt.block_on(self.memory.should_reject_write()) {
            return;
        }

        let old_value = self.data.get(key).cloned();
        
        let stored = StoredValue {
            data: value,
            expire_at,
        };

        if memory_enabled {
            let memory_delta = stored.estimated_size() - old_value.as_ref().map(|v| v.estimated_size()).unwrap_or(0);
            
            rt.block_on(async {
                if memory_delta > 0 {
                    while self.memory.check_eviction_needed().await {
                        if let Some(evicted_key) = self.memory.evict_one(&|k| self.data.get(k).cloned()).await {
                            if let Some(evicted) = self.data.remove(&evicted_key) {
                                self.expiration.cancel_expiration(&evicted_key);
                                self.memory.remove_memory(&evicted_key, &evicted).await;
                            }
                        } else {
                            break;
                        }
                    }
                }
            });
        }

        if let Some(ref old) = old_value {
            if old.expire_at.is_some() {
                self.expiration.cancel_expiration(key);
            }
            if memory_enabled {
                rt.block_on(self.memory.remove_memory(key, old));
            }
        }

        self.data.insert(key.to_string(), stored);

        if let Some(at) = expire_at {
            self.expiration.schedule_expiration(key.to_string(), at);
        }
        
        if memory_enabled {
            rt.block_on(async {
                if let Some(new_value) = self.data.get(key) {
                    self.memory.add_memory(key, &new_value).await;
                }
            });
        }
    }

    pub fn get(&self, key: &str) -> Option<StoredValue> {
        let value = self.data.get(key).map(|v| v.clone());
        
        if value.is_some() {
            let rt = tokio::runtime::Handle::current();
            if rt.block_on(self.memory.is_enabled()) {
                rt.block_on(self.memory.record_read(key));
            }
        }
        
        value
    }

    pub fn remove(&self, key: &str) -> bool {
        if let Some(old) = self.data.get(key) {
            let rt = tokio::runtime::Handle::current();
            if rt.block_on(self.memory.is_enabled()) {
                rt.block_on(self.memory.remove_memory(key, &old));
            }
        }
        
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

    pub async fn set_maxmemory(&self, maxmemory: usize) {
        self.memory.set_maxmemory(Some(maxmemory)).await;
    }

    pub async fn set_maxmemory_policy(&self, policy: MaxMemoryPolicy) {
        self.memory.set_maxmemory_policy(policy).await;
    }

    pub async fn get_maxmemory(&self) -> Option<usize> {
        self.memory.get_maxmemory().await
    }

    pub async fn get_maxmemory_policy(&self) -> MaxMemoryPolicy {
        self.memory.get_maxmemory_policy().await
    }

    pub fn current_memory_usage(&self) -> usize {
        self.memory.current_memory()
    }
}

impl Default for StorageEngine {
    fn default() -> Self {
        Self::new()
    }
}
