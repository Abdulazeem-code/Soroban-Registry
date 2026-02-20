use std::time::Instant;
use std::sync::Arc;
use sqlx::PgPool;
use crate::cache::{CacheLayer, CacheConfig};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub started_at: Instant,
    pub cache: Arc<CacheLayer>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        let config = CacheConfig::from_env();
        Self {
            db,
            started_at: Instant::now(),
            cache: Arc::new(CacheLayer::new(config)),
        }
    }
}
