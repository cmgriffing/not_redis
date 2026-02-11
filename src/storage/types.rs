use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::time::Instant;

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
        match self.expire_at {
            Some(at) => Instant::now() >= at,
            None => false,
        }
    }
}
