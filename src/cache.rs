use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const MAX_ENTRIES: usize = 10_000;
const SWEEP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3600);

struct CacheEntry {
    value: Value,
    expires_at: Option<Instant>,
}

pub struct Cache {
    store: Mutex<HashMap<String, CacheEntry>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let mut store = self.store.lock().unwrap();
        let entry = store.get(key)?;
        if let Some(expires_at) = entry.expires_at {
            if Instant::now() > expires_at {
                store.remove(key);
                return None;
            }
        }
        Some(entry.value.clone())
    }

    pub fn set(&self, key: &str, value: Value, ttl_seconds: Option<u64>) {
        let mut store = self.store.lock().unwrap();
        if store.len() >= MAX_ENTRIES && !store.contains_key(key) {
            // Remove the first (oldest inserted) entry
            if let Some(oldest_key) = store.keys().next().cloned() {
                store.remove(&oldest_key);
            }
        }
        store.insert(
            key.to_string(),
            CacheEntry {
                value,
                expires_at: ttl_seconds.map(|s| Instant::now() + std::time::Duration::from_secs(s)),
            },
        );
    }

    fn sweep(&self) {
        let mut store = self.store.lock().unwrap();
        let now = Instant::now();
        store.retain(|_, entry| match entry.expires_at {
            Some(expires_at) => now <= expires_at,
            None => true,
        });
    }
}

pub fn start_sweep(cache: Arc<Cache>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(SWEEP_INTERVAL);
        loop {
            interval.tick().await;
            cache.sweep();
        }
    });
}
