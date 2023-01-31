use std::collections::HashMap;
use std::time::{Duration, Instant};

// Expiery object is not really necessary, but in the future
// if more information needs to be saved about a request
// for example WHO is requesting, it can be added more easily
#[derive(PartialEq, Eq, Hash)]
pub struct CacheRequest {
    pub path: String,
    pub query_string: String,
    // Can add more here in the future:
    // body..
    // requester..
}

impl CacheRequest {
    pub fn new(path: String, query_string: String) -> Self {
        Self { path, query_string }
    }
}

// Data structure to hold elements with a time to live
// Basically a hashmap under the hood
pub struct Cache<K, V> {
    ttl: Duration,
    map: HashMap<K, (V, Instant)>,
}

// Make sure that the key implements comparison and hashing
impl<K: std::cmp::Eq + std::hash::Hash, V> Cache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            map: HashMap::new(),
        }
    }

    // Inserts into hashmap and computes the expiery of the object
    pub fn insert(&mut self, key: K, value: V) {
        let expires = Instant::now() + self.ttl;
        self.map.insert(key, (value, expires));
    }

    // Get is a standard get from the hashmap that does a check
    // as to whether the entry is expired
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).and_then(|(v, expires)| {
            if expires > &Instant::now() {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn remove_expired(&mut self) {
        self.map
            .retain(|_k, (_v, expires)| Instant::now() >= *expires);
    }
}
