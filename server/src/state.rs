use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{config::Config, rpc::ZcashRpcQuorum, store::SqliteStore, types::NetworkStats};

const STATS_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub config: Config,
    pub store: SqliteStore,
    pub rpc: ZcashRpcQuorum,
    // Cached trusted tip height (refreshed by the scheduler). None until first scheduler tick.
    pub trusted_tip: Mutex<Option<u64>>,
    // Network-stats cache. The 5-COUNT aggregate over 200K+ rows takes ~20s,
    // so we serve a 30s-stale snapshot to the public counter instead.
    pub network_stats_cache: Mutex<Option<(Instant, NetworkStats)>>,
}

impl AppState {
    pub fn new(config: Config, store: SqliteStore, rpc: ZcashRpcQuorum) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                store,
                rpc,
                trusted_tip: Mutex::new(None),
                network_stats_cache: Mutex::new(None),
            }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn store(&self) -> &SqliteStore {
        &self.inner.store
    }

    pub fn rpc(&self) -> &ZcashRpcQuorum {
        &self.inner.rpc
    }

    pub async fn trusted_tip(&self) -> Option<u64> {
        *self.inner.trusted_tip.lock().await
    }

    pub async fn set_trusted_tip(&self, height: u64) {
        *self.inner.trusted_tip.lock().await = Some(height);
    }

    // Returns a cached NetworkStats if the last refresh was within
    // STATS_CACHE_TTL; otherwise None. Cheap mutex lookup, no DB hit.
    pub async fn cached_network_stats(&self) -> Option<NetworkStats> {
        let guard = self.inner.network_stats_cache.lock().await;
        match &*guard {
            Some((at, stats)) if at.elapsed() < STATS_CACHE_TTL => Some(stats.clone()),
            _ => None,
        }
    }

    pub async fn store_network_stats(&self, stats: NetworkStats) {
        *self.inner.network_stats_cache.lock().await = Some((Instant::now(), stats));
    }
}
