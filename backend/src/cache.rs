use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A simple TTL-based in-memory cache for reference data.
pub struct Cache<V: Clone> {
    entries: RwLock<HashMap<String, CacheEntry<V>>>,
    ttl: Duration,
}

struct CacheEntry<V: Clone> {
    value: V,
    inserted_at: Instant,
}

impl<V: Clone + Send + Sync> Cache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        let entries = self.entries.read().await;
        entries.get(key).and_then(|entry| {
            if entry.inserted_at.elapsed() < self.ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub async fn insert(&self, key: String, value: V) {
        let mut entries = self.entries.write().await;
        entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
    }

    pub async fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
}

pub type SharedCache<V> = Arc<Cache<V>>;

pub fn new_shared_cache<V: Clone + Send + Sync>(ttl: Duration) -> SharedCache<V> {
    Arc::new(Cache::new(ttl))
}

/// Items cache stores 商品 master data (SKU list, item lookups).
/// TTL: 5 minutes — item definitions rarely change.
pub type ItemsCache = Cache<serde_json::Value>;

/// Location cache stores warehouse location lists and hierarchy data.
/// TTL: 2 minutes — locations can be added/removed more frequently.
pub type LocationCache = Cache<serde_json::Value>;

/// Dashboard cache stores aggregated statistics (item counts, order counts, etc.).
/// TTL: 30 seconds — dashboard is hit frequently but data freshness matters.
pub type DashboardCache = Cache<serde_json::Value>;

/// Holds all application caches. Created once at startup, injected via Extension layer.
#[derive(Clone)]
pub struct CacheManager {
    pub items: Arc<ItemsCache>,
    pub locations: Arc<LocationCache>,
    pub dashboard: Arc<DashboardCache>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Cache::new(Duration::from_secs(300))),    // 5 min
            locations: Arc::new(Cache::new(Duration::from_secs(120))), // 2 min
            dashboard: Arc::new(Cache::new(Duration::from_secs(30))),  // 30 sec
        }
    }

    /// Invalidate all caches. Call after data mutations (create/update/delete).
    pub async fn invalidate_all(&self) {
        self.items.clear().await;
        self.locations.clear().await;
        self.dashboard.clear().await;
    }

    /// Invalidate item-related caches.
    pub async fn invalidate_items(&self) {
        self.items.clear().await;
    }

    /// Invalidate location-related caches.
    pub async fn invalidate_locations(&self) {
        self.locations.clear().await;
    }

    /// Invalidate dashboard statistics cache.
    pub async fn invalidate_dashboard(&self) {
        self.dashboard.clear().await;
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}
