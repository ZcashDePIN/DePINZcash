use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{
    config::Config,
    rpc::ZcashRpcQuorum,
    store::SqliteStore,
    types::{NetworkStats, Node, Proof, WalletStats},
};

const STATS_CACHE_TTL: Duration = Duration::from_secs(30);
const LIST_CACHE_TTL: Duration = Duration::from_secs(30);

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
    // Per-(limit) leaderboard cache. Same reasoning: GROUP BY wallet over
    // tens of thousands of rows is expensive, public counter is fine with
    // 30s staleness.
    pub leaderboard_cache: Mutex<HashMap<i64, (Instant, Vec<WalletStats>)>>,
    // Per-(limit) active-nodes cache for the /explorer page.
    pub active_nodes_cache: Mutex<HashMap<i64, (Instant, Vec<Node>)>>,
    // Recent-proofs cache (keyed by limit; verdict/wallet filters bypass).
    pub recent_proofs_cache: Mutex<HashMap<i64, (Instant, Vec<Proof>)>>,
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
                leaderboard_cache: Mutex::new(HashMap::new()),
                active_nodes_cache: Mutex::new(HashMap::new()),
                recent_proofs_cache: Mutex::new(HashMap::new()),
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

    pub async fn cached_leaderboard(&self, limit: i64) -> Option<Vec<WalletStats>> {
        let guard = self.inner.leaderboard_cache.lock().await;
        match guard.get(&limit) {
            Some((at, board)) if at.elapsed() < LIST_CACHE_TTL => Some(board.clone()),
            _ => None,
        }
    }
    pub async fn store_leaderboard(&self, limit: i64, board: Vec<WalletStats>) {
        self.inner
            .leaderboard_cache
            .lock()
            .await
            .insert(limit, (Instant::now(), board));
    }

    pub async fn cached_active_nodes(&self, limit: i64) -> Option<Vec<Node>> {
        let guard = self.inner.active_nodes_cache.lock().await;
        match guard.get(&limit) {
            Some((at, nodes)) if at.elapsed() < LIST_CACHE_TTL => Some(nodes.clone()),
            _ => None,
        }
    }
    pub async fn store_active_nodes(&self, limit: i64, nodes: Vec<Node>) {
        self.inner
            .active_nodes_cache
            .lock()
            .await
            .insert(limit, (Instant::now(), nodes));
    }

    pub async fn cached_recent_proofs(&self, limit: i64) -> Option<Vec<Proof>> {
        let guard = self.inner.recent_proofs_cache.lock().await;
        match guard.get(&limit) {
            Some((at, proofs)) if at.elapsed() < LIST_CACHE_TTL => Some(proofs.clone()),
            _ => None,
        }
    }
    pub async fn store_recent_proofs(&self, limit: i64, proofs: Vec<Proof>) {
        self.inner
            .recent_proofs_cache
            .lock()
            .await
            .insert(limit, (Instant::now(), proofs));
    }
}
