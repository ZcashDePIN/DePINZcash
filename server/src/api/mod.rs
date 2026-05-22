pub mod admin;
pub mod challenges;
pub mod health;
pub mod nodes;
pub mod proofs;
pub mod rewards;
pub mod stats;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::ConnectInfo,
    http::{HeaderName, HeaderValue, Method, Request},
    routing::{get, post},
    Router,
};
use tower_governor::{
    errors::GovernorError, governor::GovernorConfigBuilder, key_extractor::KeyExtractor,
    GovernorLayer,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

// Behind Fly's edge, the TCP peer is always the proxy — so the default
// PeerIpKeyExtractor collapses every visitor into one global bucket. Fly
// passes the real client IP in `Fly-Client-IP`. Fall back to the first
// X-Forwarded-For entry, then the connect-info peer for local dev.
#[derive(Debug, Clone, Copy)]
struct FlyClientIpKeyExtractor;

impl KeyExtractor for FlyClientIpKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if let Some(ip) = req
            .headers()
            .get("fly-client-ip")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            return Ok(ip.to_string());
        }
        if let Some(xff) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(first) = xff.split(',').next().map(|s| s.trim()).filter(|s| !s.is_empty()) {
                return Ok(first.to_string());
            }
        }
        if let Some(ci) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
            return Ok(ci.0.ip().to_string());
        }
        Err(GovernorError::UnableToExtractKey)
    }
}

pub fn router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // Hot mutating endpoints get a per-IP rate limit. Read-only endpoints stay open.
    let mut posts: Router<AppState> = Router::new()
        .route("/api/nodes/register", post(nodes::register))
        .route("/api/proofs/submit", post(proofs::submit))
        .route("/api/challenges/request", post(challenges::request))
        .route("/api/challenges/submit", post(challenges::submit))
        .route("/api/admin/snapshot/publish", post(admin::publish_snapshot))
        .route("/api/admin/nodes/:id/purge", post(admin::purge_node))
        .route("/api/admin/nodes/:id/suspend", post(admin::suspend_node));

    if state.config().rate_limit_enabled {
        let gov_conf = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(state.config().rate_limit_per_second)
                .burst_size(state.config().rate_limit_burst)
                .key_extractor(FlyClientIpKeyExtractor)
                .finish()
                .expect("governor config builds with positive values"),
        );
        posts = posts.layer(GovernorLayer { config: gov_conf });
    }

    let gets: Router<AppState> = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/api/info", get(health::info))
        .route("/api/nodes", get(nodes::list_active))
        .route("/api/nodes/:id", get(nodes::get_by_id))
        .route("/api/nodes/:id/proofs", get(nodes::list_proofs))
        .route("/api/nodes/:id/series", get(nodes::daily_series))
        .route("/api/proofs/recent", get(proofs::list_recent))
        .route("/api/wallet/:wallet/nodes", get(nodes::list_for_wallet))
        .route("/api/wallet/:wallet/stats", get(stats::wallet_stats))
        .route("/api/wallet/:wallet/proofs", get(proofs::list_for_wallet))
        .route("/api/wallet/:wallet/claim/latest", get(rewards::latest_claim))
        .route("/api/stats/network", get(stats::network))
        .route("/api/stats/leaderboard", get(stats::leaderboard))
        .route("/api/snapshots/latest", get(rewards::latest_snapshot));

    gets.merge(posts)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn build_cors(state: &AppState) -> CorsLayer {
    let origins = &state.config().cors_allowed_origins;
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-admin-key"),
        ]);
    if origins.is_empty() {
        base
    } else {
        let list: Vec<HeaderValue> = origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        base.allow_origin(AllowOrigin::list(list))
    }
}
