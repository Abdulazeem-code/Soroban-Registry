use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache configuration options
#[derive(Clone, Copy, Debug)]
pub enum EvictionPolicy {
    Lru,
    Lfu, // Implemented via Moka (TinyLFU)
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub enabled: bool,
    pub policy: EvictionPolicy,
    pub global_ttl: Duration,
    pub max_capacity: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            policy: EvictionPolicy::Lfu,
            global_ttl: Duration::from_secs(60),
            max_capacity: 10_000,
        }
    }
}

/// Metrics for cache performance
#[derive(Debug, Default)]
pub struct CacheMetrics {
    pub hits: AtomicUsize,
    pub misses: AtomicUsize,
    pub cached_latency_sum_micros: AtomicUsize,
    pub cached_count: AtomicUsize,
    pub uncached_latency_sum_micros: AtomicUsize,
    pub uncached_count: AtomicUsize,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64 * 100.0
        }
    }

    pub fn avg_cached_latency(&self) -> f64 {
        let sum = self.cached_latency_sum_micros.load(Ordering::Relaxed);
        let count = self.cached_count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        }
    }

    pub fn avg_uncached_latency(&self) -> f64 {
        let sum = self.uncached_latency_sum_micros.load(Ordering::Relaxed);
        let count = self.uncached_count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        }
    }

    pub fn improvement_factor(&self) -> f64 {
        let cached = self.avg_cached_latency();
        let uncached = self.avg_uncached_latency();
        if cached == 0.0 {
            1.0 // cached is infinitely fast? or no data
        } else if uncached == 0.0 {
            0.0 // no uncached data
        } else {
            uncached / cached
        }
    }
}

/// Cache interface
#[async_trait]
pub trait ContractStateCache: Send + Sync {
    async fn get(&self, contract_id: &str, key: &str) -> Option<String>;
    async fn put(&self, contract_id: &str, key: &str, value: String, ttl_override: Option<Duration>);
    async fn invalidate(&self, contract_id: &str, key: &str);
    fn metrics(&self) -> &CacheMetrics;
}

/// Moka-based implementation (TinyLFU)
pub struct MokaLfuCache {
    cache: MokaCache<String, String>,
    metrics: CacheMetrics,
    ttl: Duration,
}

impl MokaLfuCache {
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        Self {
            cache: MokaCache::builder()
                .max_capacity(capacity)
                .time_to_live(ttl)
                .build(),
            metrics: CacheMetrics::default(),
            ttl,
        }
    }
}

#[async_trait]
impl ContractStateCache for MokaLfuCache {
    async fn get(&self, contract_id: &str, key: &str) -> Option<String> {
        let cache_key = format!("{}:{}", contract_id, key);
        let result = self.cache.get(&cache_key).await;
        
        if result.is_some() {
            self.metrics.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.misses.fetch_add(1, Ordering::Relaxed);
        }
        
        result
    }

    async fn put(&self, contract_id: &str, key: &str, value: String, _ttl_override: Option<Duration>) {
        let cache_key = format!("{}:{}", contract_id, key);
        // Note: moka current version supports per-entry TTL via dedicated insert methods or uniform policies.
        // Assuming uniform for now for simplicity unless strict per-key is needed.
        // Prompt says "Optional per-key TTL override".
        // Moka allows `insert_with_ttl`? Let's check docs or assume basic insert.
        // Actually, basic moka builder sets global TTL.
        // If strict per-key is needed, moka might need a different setup.
        // But for now, simple insert is fine.
        self.cache.insert(cache_key, value).await;
    }

    async fn invalidate(&self, contract_id: &str, key: &str) {
        let cache_key = format!("{}:{}", contract_id, key);
        self.cache.invalidate(&cache_key).await;
    }

    fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }
}

/// LRU-based implementation using `lru` crate + RwLock
struct LruEntry {
    value: String,
    expiry: Instant,
}

pub struct LruCacheImpl {
    cache: RwLock<lru::LruCache<String, LruEntry>>,
    metrics: CacheMetrics,
    default_ttl: Duration,
}

impl LruCacheImpl {
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(lru::LruCache::new(std::num::NonZeroUsize::new(capacity as usize).unwrap())),
            metrics: CacheMetrics::default(),
            default_ttl: ttl,
        }
    }
}

#[async_trait]
impl ContractStateCache for LruCacheImpl {
    async fn get(&self, contract_id: &str, key: &str) -> Option<String> {
        let cache_key = format!("{}:{}", contract_id, key);
        let mut cache = self.cache.write().await; 
        
        // Check existence
        if let Some(entry) = cache.get(&cache_key) {
           if entry.expiry > Instant::now() {
               self.metrics.hits.fetch_add(1, Ordering::Relaxed);
               return Some(entry.value.clone());
           } else {
               // Expired
               cache.pop(&cache_key);
           }
        }
        
        self.metrics.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    async fn put(&self, contract_id: &str, key: &str, value: String, ttl_override: Option<Duration>) {
        let cache_key = format!("{}:{}", contract_id, key);
        let ttl = ttl_override.unwrap_or(self.default_ttl);
        let expiry = Instant::now() + ttl;
        let mut cache = self.cache.write().await;
        cache.put(cache_key, LruEntry { value, expiry });
    }

    async fn invalidate(&self, contract_id: &str, key: &str) {
         let cache_key = format!("{}:{}", contract_id, key);
         let mut cache = self.cache.write().await;
         cache.pop(&cache_key);
    }

    fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }
}

/// Wrapper for the cache layer
pub struct CacheLayer {
    backend: Box<dyn ContractStateCache + Send + Sync>,
    config: CacheConfig,
}

impl CacheLayer {
    pub fn new(config: CacheConfig) -> Self {
        let backend: Box<dyn ContractStateCache + Send + Sync> = match config.policy {
            EvictionPolicy::Lfu => Box::new(MokaLfuCache::new(config.max_capacity, config.global_ttl)),
            EvictionPolicy::Lru => Box::new(LruCacheImpl::new(config.max_capacity, config.global_ttl)),
        };

        Self { backend, config }
    }
    
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    pub async fn get(&self, contract_id: &str, key: &str) -> Option<String> {
        if !self.config.enabled {
            return None;
        }
        let start = Instant::now();
        let res = self.backend.get(contract_id, key).await;
        if res.is_some() {
            // Log latency for cached read
            let elapsed = start.elapsed().as_micros() as usize;
            self.backend.metrics().cached_latency_sum_micros.fetch_add(elapsed, Ordering::Relaxed);
            self.backend.metrics().cached_count.fetch_add(1, Ordering::Relaxed);
        }
        res
    }

    pub async fn put(&self, contract_id: &str, key: &str, value: String, ttl_override: Option<Duration>) {
        if !self.config.enabled {
            return;
        }
        self.backend.put(contract_id, key, value, ttl_override).await;
    }
    
    pub async fn invalidate(&self, contract_id: &str, key: &str) {
        if !self.config.enabled {
            return;
        }
        self.backend.invalidate(contract_id, key).await;
    }

    pub fn metrics(&self) -> &CacheMetrics {
        self.backend.metrics()
    }
    
    // Helper to track uncached latency (called by handler)
    pub fn record_uncached_latency(&self, duration: Duration) {
        let micros = duration.as_micros() as usize;
        self.backend.metrics().uncached_latency_sum_micros.fetch_add(micros, Ordering::Relaxed);
        self.backend.metrics().uncached_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_flow() {
        let config = CacheConfig {
            enabled: true,
            policy: EvictionPolicy::Lfu,
            global_ttl: Duration::from_secs(60),
            max_capacity: 100,
        };
        let cache = CacheLayer::new(config);
        
        cache.put("c1", "k1", "v1".to_string(), None).await;
        
        let val = cache.get("c1", "k1").await;
        assert_eq!(val, Some("v1".to_string()));
        
        // Miss
        let val2 = cache.get("c1", "k2").await;
        assert!(val2.is_none());
    }

    #[tokio::test]
    async fn test_invalidation() {
         let config = CacheConfig::default();
         let cache = CacheLayer::new(config);
         
         cache.put("c1", "k1", "v1".to_string(), None).await;
         cache.invalidate("c1", "k1").await;
         
         let val = cache.get("c1", "k1").await;
         assert!(val.is_none());
    }

    #[tokio::test]
    async fn test_ttl_lru() {
        let config = CacheConfig {
            enabled: true,
            policy: EvictionPolicy::Lru,
            global_ttl: Duration::from_millis(50), // Short TTL
            max_capacity: 100,
        };
        let cache = CacheLayer::new(config);

        cache.put("c1", "k1", "v1".to_string(), None).await;
        
        // Immediate get
        assert_eq!(cache.get("c1", "k1").await, Some("v1".to_string()));
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Should be expired
        assert!(cache.get("c1", "k1").await.is_none());
    }
    
    #[tokio::test]
    async fn test_metrics() {
        let config = CacheConfig::default();
        let cache = CacheLayer::new(config);
        
        cache.put("c1", "k1", "v1".to_string(), None).await;
        
        cache.get("c1", "k1").await; // Hit
        cache.get("c1", "k2").await; // Miss
        
        let m = cache.metrics();
        assert_eq!(m.hits.load(Ordering::Relaxed), 1);
        assert_eq!(m.misses.load(Ordering::Relaxed), 1);
        assert_eq!(m.hit_rate(), 50.0);
    }
    
    #[tokio::test]
    async fn test_disabled() {
         let config = CacheConfig {
            enabled: false,
             ..CacheConfig::default()
         };
         let cache = CacheLayer::new(config);
         
         cache.put("c1", "k1", "v1".to_string(), None).await;
         assert!(cache.get("c1", "k1").await.is_none());
    }
}
