use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaxMemoryPolicy {
    #[default]
    NoEviction,
    AllKeysLru,
    AllKeysRandom,
    AllKeysLfu,
    VolatileLru,
    VolatileRandom,
    VolatileTtl,
    VolatileLfu,
}

impl MaxMemoryPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            MaxMemoryPolicy::NoEviction => "noeviction",
            MaxMemoryPolicy::AllKeysLru => "allkeys-lru",
            MaxMemoryPolicy::AllKeysRandom => "allkeys-random",
            MaxMemoryPolicy::AllKeysLfu => "allkeys-lfu",
            MaxMemoryPolicy::VolatileLru => "volatile-lru",
            MaxMemoryPolicy::VolatileRandom => "volatile-random",
            MaxMemoryPolicy::VolatileTtl => "volatile-ttl",
            MaxMemoryPolicy::VolatileLfu => "volatile-lfu",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "noeviction" => Some(MaxMemoryPolicy::NoEviction),
            "allkeys-lru" => Some(MaxMemoryPolicy::AllKeysLru),
            "allkeys-random" => Some(MaxMemoryPolicy::AllKeysRandom),
            "allkeys-lfu" => Some(MaxMemoryPolicy::AllKeysLfu),
            "volatile-lru" => Some(MaxMemoryPolicy::VolatileLru),
            "volatile-random" => Some(MaxMemoryPolicy::VolatileRandom),
            "volatile-ttl" => Some(MaxMemoryPolicy::VolatileTtl),
            "volatile-lfu" => Some(MaxMemoryPolicy::VolatileLfu),
            _ => None,
        }
    }

    pub fn is_volatile(&self) -> bool {
        matches!(
            self,
            MaxMemoryPolicy::VolatileLru
                | MaxMemoryPolicy::VolatileRandom
                | MaxMemoryPolicy::VolatileTtl
                | MaxMemoryPolicy::VolatileLfu
        )
    }
}

impl fmt::Display for MaxMemoryPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub maxmemory: Option<usize>,
    pub maxmemory_policy: MaxMemoryPolicy,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            maxmemory: None,
            maxmemory_policy: MaxMemoryPolicy::NoEviction,
        }
    }
}

impl StorageConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_maxmemory(mut self, maxmemory: usize) -> Self {
        self.maxmemory = Some(maxmemory);
        self
    }

    pub fn with_maxmemory_policy(mut self, policy: MaxMemoryPolicy) -> Self {
        self.maxmemory_policy = policy;
        self
    }
}
