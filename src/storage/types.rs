//! Internal data types for the storage engine.

use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, VecDeque};
use std::time::Instant;

/// Internal data types stored in the engine.
///
/// These represent the actual data structures that can be stored,
/// as opposed to the RESP protocol [`Value`](crate::Value) types.
#[derive(Debug, Clone)]
pub enum RedisData {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(FxHashSet<Vec<u8>>),
    Hash(FxHashMap<Vec<u8>, Vec<u8>>),
    ZSet(BTreeMap<Vec<u8>, f64>),
}

/// A value stored in the storage engine with optional expiration.
#[derive(Debug, Clone)]
pub struct StoredValue {
    pub data: RedisData,
    pub expire_at: Option<Instant>,
}

impl StoredValue {
    pub fn is_expired(&self) -> bool {
        match self.expire_at {
            Some(at) => Instant::now() >= at,
            None => false,
        }
    }
}
