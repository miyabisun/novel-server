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
        let store = self.store.lock().unwrap();
        let entry = store.get(key)?;
        if let Some(expires_at) = entry.expires_at {
            if Instant::now() > expires_at {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_returns_none_for_missing_key() {
        let cache = Cache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn set_and_get_without_ttl() {
        let cache = Cache::new();
        cache.set("key", json!("value"), None);
        assert_eq!(cache.get("key"), Some(json!("value")));
    }

    #[test]
    fn set_and_get_with_ttl() {
        let cache = Cache::new();
        cache.set("key", json!(42), Some(3600));
        assert_eq!(cache.get("key"), Some(json!(42)));
    }

    #[test]
    fn expired_entry_returns_none() {
        let cache = Cache::new();
        // TTL of 0 seconds = already expired by the time we get
        cache.set("key", json!("old"), Some(0));
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(cache.get("key").is_none());
    }

    #[test]
    fn overwrite_existing_key() {
        let cache = Cache::new();
        cache.set("key", json!("first"), None);
        cache.set("key", json!("second"), None);
        assert_eq!(cache.get("key"), Some(json!("second")));
    }

    #[test]
    fn no_ttl_entry_never_expires() {
        let cache = Cache::new();
        cache.set("key", json!("forever"), None);
        // Even after sweep, no-TTL entries remain
        cache.sweep();
        assert_eq!(cache.get("key"), Some(json!("forever")));
    }

    #[test]
    fn sweep_removes_expired_entries() {
        let cache = Cache::new();
        cache.set("expired", json!("old"), Some(0));
        cache.set("alive", json!("new"), None);
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.sweep();
        assert!(cache.get("expired").is_none());
        assert_eq!(cache.get("alive"), Some(json!("new")));
    }

    #[test]
    fn evicts_when_at_capacity() {
        let cache = Cache::new();
        for i in 0..MAX_ENTRIES {
            cache.set(&format!("k{i}"), json!(i), None);
        }
        // Adding one more should evict one
        cache.set("overflow", json!("new"), None);
        let store = cache.store.lock().unwrap();
        assert_eq!(store.len(), MAX_ENTRIES);
        assert!(store.contains_key("overflow"));
    }

    #[test]
    fn update_existing_key_at_capacity_does_not_evict() {
        let cache = Cache::new();
        for i in 0..MAX_ENTRIES {
            cache.set(&format!("k{i}"), json!(i), None);
        }
        // Updating existing key should not evict
        cache.set("k0", json!("updated"), None);
        let store = cache.store.lock().unwrap();
        assert_eq!(store.len(), MAX_ENTRIES);
        assert_eq!(store.get("k0").unwrap().value, json!("updated"));
    }

    #[test]
    fn stores_various_json_types() {
        let cache = Cache::new();
        cache.set("string", json!("text"), None);
        cache.set("number", json!(123), None);
        cache.set("array", json!([1, 2, 3]), None);
        cache.set("object", json!({"a": 1}), None);
        cache.set("null", json!(null), None);

        assert_eq!(cache.get("string"), Some(json!("text")));
        assert_eq!(cache.get("number"), Some(json!(123)));
        assert_eq!(cache.get("array"), Some(json!([1, 2, 3])));
        assert_eq!(cache.get("object"), Some(json!({"a": 1})));
        assert_eq!(cache.get("null"), Some(json!(null)));
    }
}
