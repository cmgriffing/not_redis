//! Key expiration management.

use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;

/// Manages key expiration for the storage engine.
/// 
/// Uses a time-ordered map to efficiently find expired keys
/// and a background task to periodically sweep and remove them.
pub struct ExpirationManager {
    expirations: Arc<Mutex<BTreeMap<Instant, HashSet<String>>>>,
    sweep_interval: Duration,
}

impl ExpirationManager {
    pub fn new(sweep_interval_ms: u64) -> Self {
        Self {
            expirations: Arc::new(Mutex::new(BTreeMap::new())),
            sweep_interval: Duration::from_millis(sweep_interval_ms),
        }
    }

    pub fn schedule_expiration(&self, key: String, expire_at: Instant) {
        let mut expirations = self.expirations.lock().unwrap();
        let entry = expirations.entry(expire_at).or_insert_with(HashSet::new);
        entry.insert(key);
    }

    pub fn cancel_expiration(&self, key: &str) {
        let mut expirations = self.expirations.lock().unwrap();
        for (_, keys) in expirations.iter_mut() {
            keys.remove(key);
        }
    }

    pub async fn start_sweep_task<F>(&self, mut remove_fn: F)
    where
        F: FnMut(&str) + Send + Sync + 'static,
    {
        let mut interval = time::interval(self.sweep_interval);
        loop {
            interval.tick().await;
            let now = Instant::now();

            let mut expirations = self.expirations.lock().unwrap();
            let mut keys_to_remove = Vec::new();

            let expired_times: Vec<Instant> = expirations
                .iter()
                .filter(|(t, _)| **t <= now)
                .map(|(t, _)| *t)
                .collect();

            for t in expired_times {
                if let Some(keys) = expirations.remove(&t) {
                    for key in keys {
                        keys_to_remove.push(key);
                    }
                }
            }
            drop(expirations);

            for key in keys_to_remove {
                self.cancel_expiration(&key);
                remove_fn(&key);
            }
        }
    }

    pub fn clear_all(&self) {
        let mut expirations = self.expirations.lock().unwrap();
        expirations.clear();
    }
}

impl Clone for ExpirationManager {
    fn clone(&self) -> Self {
        Self {
            expirations: Arc::clone(&self.expirations),
            sweep_interval: self.sweep_interval,
        }
    }
}
