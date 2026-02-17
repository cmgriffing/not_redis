//! Storage engine and related types for the Redis-like store.

pub mod types;
pub mod engine;
pub mod expire;
pub mod config;
pub mod memory;

pub use types::{RedisData, StoredValue};
pub use engine::StorageEngine;
pub use expire::ExpirationManager;
pub use config::{MaxMemoryPolicy, StorageConfig};
