use crate::storage::config::{MaxMemoryPolicy, StorageConfig};
use crate::storage::types::StoredValue;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

const KEY_OVERHEAD: usize = 50;
const COUNTER_INIT: u32 = 5;

#[derive(Debug, Clone)]
pub struct MemoryTracker {
    config: Arc<tokio::sync::RwLock<StorageConfig>>,
    total_memory: AtomicUsize,
    lru_order: Arc<tokio::sync::RwLock<VecDeque<String>>>,
    access_counts: Arc<tokio::sync::RwLock<std::collections::HashMap<String, u32>>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            config: Arc::new(tokio::sync::RwLock::new(StorageConfig::new())),
            total_memory: AtomicUsize::new(0),
            lru_order: Arc::new(tokio::sync::RwLock::new(VecDeque::new())),
            access_counts: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn set_maxmemory(&self, maxmemory: Option<usize>) {
        let mut config = self.config.write().await;
        config.maxmemory = maxmemory;
    }

    pub async fn set_maxmemory_policy(&self, policy: MaxMemoryPolicy) {
        let mut config = self.config.write().await;
        config.maxmemory_policy = policy;
    }

    pub async fn get_maxmemory(&self) -> Option<usize> {
        let config = self.config.read().await;
        config.maxmemory
    }

    pub async fn get_maxmemory_policy(&self) -> MaxMemoryPolicy {
        let config = self.config.read().await;
        config.maxmemory_policy
    }

    pub fn current_memory(&self) -> usize {
        self.total_memory.load(Ordering::Relaxed)
    }

    pub async fn is_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.maxmemory.is_some()
    }

    pub async fn add_memory(&self, key: &str, value: &StoredValue) -> usize {
        let size = value.data.estimated_size() + key.len() + KEY_OVERHEAD;
        self.total_memory.fetch_add(size, Ordering::Relaxed);
        self.update_access(key, value.expire_at.is_some()).await;
        size
    }

    pub async fn remove_memory(&self, key: &str, value: &StoredValue) {
        let size = value.data.estimated_size() + key.len() + KEY_OVERHEAD;
        self.total_memory.fetch_sub(size, Ordering::Relaxed);
        self.remove_from_tracking(key).await;
    }

    pub async fn check_eviction_needed(&self) -> bool {
        let config = self.config.read().await;
        if let Some(limit) = config.maxmemory {
            self.total_memory.load(Ordering::Relaxed) >= limit && config.maxmemory_policy != MaxMemoryPolicy::NoEviction
        } else {
            false
        }
    }

    pub async fn should_reject_write(&self) -> bool {
        let config = self.config.read().await;
        if let Some(limit) = config.maxmemory {
            self.total_memory.load(Ordering::Relaxed) >= limit && config.maxmemory_policy == MaxMemoryPolicy::NoEviction
        } else {
            false
        }
    }

    async fn update_access(&self, key: &str, has_ttl: bool) {
        let policy = self.get_maxmemory_policy().await;
        
        if matches!(policy, MaxMemoryPolicy::AllKeysLru | MaxMemoryPolicy::VolatileLru) {
            let mut lru = self.lru_order.write().await;
            lru.retain(|k| k != key);
            lru.push_back(key.to_string());
        }

        if matches!(policy, MaxMemoryPolicy::AllKeysLfu | MaxMemoryPolicy::VolatileLfu) {
            let mut counts = self.access_counts.write().await;
            let counter = counts.entry(key.to_string()).or_insert(COUNTER_INIT);
            *counter = counter.saturating_add(1);
        }
    }

    async fn remove_from_tracking(&self, key: &str) {
        let mut lru = self.lru_order.write().await;
        lru.retain(|k| k != key);
        
        let mut counts = self.access_counts.write().await;
        counts.remove(key);
    }

    pub async fn evict_one<F>(&self, get_value: &F) -> Option<String>
    where
        F: Fn(&str) -> Option<StoredValue>,
    {
        let policy = self.get_maxmemory_policy().await;
        
        match policy {
            MaxMemoryPolicy::NoEviction => None,
            
            MaxMemoryPolicy::AllKeysRandom | MaxMemoryPolicy::VolatileRandom => {
                self.evict_random(get_value, policy.is_volatile()).await
            }
            
            MaxMemoryPolicy::AllKeysLru | MaxMemoryPolicy::VolatileLru => {
                self.evict_lru(get_value, policy.is_volatile()).await
            }
            
            MaxMemoryPolicy::AllKeysLfu | MaxMemoryPolicy::VolatileLfu => {
                self.evict_lfu(get_value, policy.is_volatile()).await
            }
            
            MaxMemoryPolicy::VolatileTtl => {
                self.evict_ttl(get_value).await
            }
        }
    }

    async fn evict_random<F>(&self, get_value: &F, volatile_only: bool) -> Option<String>
    where
        F: Fn(&str) -> Option<StoredValue>,
    {
        let mut lru = self.lru_order.read().await;
        let keys: Vec<String> = lru.iter().cloned().collect();
        drop(lru);

        let candidate: Vec<String> = if volatile_only {
            keys.into_iter()
                .filter(|k| get_value(k).map(|v| v.expire_at.is_some()).unwrap_or(false))
                .collect()
        } else {
            keys
        };

        if candidate.is_empty() {
            return None;
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let idx = {
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .hash(&mut hasher);
            (hasher.finish() as usize) % candidate.len()
        };

        Some(candidate[idx].clone())
    }

    async fn evict_lru<F>(&self, get_value: &F, volatile_only: bool) -> Option<String>
    where
        F: Fn(&str) -> Option<StoredValue>,
    {
        let mut lru = self.lru_order.write().await;
        
        while let Some(key) = lru.pop_front() {
            if let Some(value) = get_value(&key) {
                if !volatile_only || value.expire_at.is_some() {
                    return Some(key);
                }
            }
        }
        
        None
    }

    async fn evict_lfu<F>(&self, get_value: &F, volatile_only: bool) -> Option<String>
    where
        F: Fn(&str) -> Option<StoredValue>,
    {
        let mut counts = self.access_counts.write().await;
        
        let min_key = counts
            .iter()
            .filter(|(k, _)| {
                if volatile_only {
                    get_value(k).map(|v| v.expire_at.is_some()).unwrap_or(false)
                } else {
                    true
                }
            })
            .min_by_key(|(_, v)| *v)
            .map(|(k, _)| k.clone());

        if let Some(key) = min_key {
            counts.remove(&key);
            return Some(key);
        }
        
        None
    }

    async fn evict_ttl<F>(&self, get_value: &F) -> Option<String>
    where
        F: Fn(&str) -> Option<StoredValue>,
    {
        let mut lru = self.lru_order.read().await;
        
        let min_key = lru
            .iter()
            .filter_map(|k| {
                get_value(k).and_then(|v| v.expire_at.map(|at| (k.clone(), at)))
            })
            .min_by_key(|(_, at)| *at)
            .map(|(k, _)| k);

        drop(lru);

        if let Some(key) = min_key {
            self.remove_from_tracking(&key).await;
            return Some(key);
        }
        
        None
    }

    pub async fn record_read(&self, key: &str) {
        self.update_access(key, false).await;
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
