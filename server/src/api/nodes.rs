use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{self, AuthError},
    error::{AppError, AppResult},
    state::AppState,
    types::{Node, NodeDailyBucket, NodeKind, NodeStatus, Proof},
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub wallet: String,
    pub signature: String,
    pub nonce: String,
    // RFC3339 string. We sign the literal bytes the client sent, never re-format
    // through chrono — chrono normalizes UTC to "+00:00" but JS toISOString()
    // uses "Z", which would break ed25519 verification byte-for-byte.
    pub timestamp: String,
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub rpc_endpoint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub node: PublicNode,
    pub auth_token: String,
}

#[derive(Debug, Serialize)]
pub struct PublicNode {
    pub id: Uuid,
    pub wallet: String,
    pub kind: String,
    pub label: Option<String>,
    pub rpc_endpoint: Option<String>,
    pub network: String,
    pub status: String,
    pub last_height: Option<u64>,
    pub last_block_hash: Option<String>,
    pub last_proof_at: Option<DateTime<Utc>>,
    pub registered_at: DateTime<Utc>,
    pub points: u64,
    pub uptime_seconds: u64,
}

impl From<&Node> for PublicNode {
    fn from(n: &Node) -> Self {
        Self {
            id: n.id,
            wallet: n.wallet.clone(),
            kind: n.kind.as_str().to_string(),
            label: n.label.clone(),
            rpc_endpoint: n.rpc_endpoint.clone(),
            network: n.network.clone(),
            status: n.status.as_str().to_string(),
            last_height: n.last_height,
            last_block_hash: n.last_block_hash.clone(),
            last_proof_at: n.last_proof_at,
            registered_at: n.registered_at,
            points: n.points,
            uptime_seconds: n.uptime_seconds,
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>> {
    if !state.config().registration_enabled {
        return Err(AppError::Forbidden);
    }
    let kind = NodeKind::parse(&req.kind)
        .ok_or_else(|| AppError::bad_request(format!("unknown node kind: {}", req.kind)))?;

    auth::check_nonce(&req.nonce).map_err(AppError::from)?;
    let ts = DateTime::parse_from_rfc3339(&req.timestamp)
        .map_err(|e| AppError::bad_request(format!("invalid timestamp: {e}")))?
        .with_timezone(&Utc);
    auth::check_timestamp(ts, state.config().max_clock_skew).map_err(AppError::from)?;

    if let Some(endpoint) = &req.rpc_endpoint {
        validate_rpc_endpoint(endpoint)?;
    }

    let label = req.label.clone().unwrap_or_default();
    let msg = auth::registration_message(
        &req.wallet,
        &req.nonce,
        &req.timestamp,
        kind.as_str(),
        state.config().network.as_str(),
        &label,
    );

    auth::verify_solana_signature(&req.wallet, &msg, &req.signature)
        .map_err(|e: AuthError| AppError::from(e))?;

    let store = state.store();
    if !store.try_use_nonce(&req.nonce, &req.wallet).await? {
        return Err(AppError::conflict("nonce already used"));
    }

    let existing = store.list_nodes_by_wallet(&req.wallet).await?;

    // Hard cap on nodes per wallet — blocks the label-spam farming pattern.
    let cap = state.config().max_nodes_per_wallet;
    if cap > 0 && existing.len() as u32 >= cap {
        return Err(AppError::Forbidden);
    }

    let label_for_uniq = req.label.clone().unwrap_or_default();
    let already: Vec<_> = existing
        .iter()
        .filter(|n| n.kind == kind && n.label.clone().unwrap_or_default() == label_for_uniq)
        .collect();
    if !already.is_empty() {
        return Err(AppError::conflict(
            "node already registered (wallet, kind, label) — use a unique label",
        ));
    }

    let node = Node {
        id: Uuid::new_v4(),
        wallet: req.wallet.clone(),
        kind,
        label: req.label.clone(),
        rpc_endpoint: req.rpc_endpoint.clone(),
        network: state.config().network.as_str().to_string(),
        status: NodeStatus::Registered,
        last_height: None,
        last_block_hash: None,
        last_proof_at: None,
        registered_at: Utc::now(),
        points: 0,
        uptime_seconds: 0,
    };

    let auth_token = auth::generate_auth_token();
    store.insert_node(&node, &auth_token).await?;

    tracing::info!(node_id = %node.id, wallet = %node.wallet, kind = %node.kind.as_str(), "node registered");

    Ok(Json(RegisterResponse {
        node: PublicNode::from(&node),
        auth_token,
    }))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PublicNode>> {
    let node = state.store().get_node(id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(PublicNode::from(&node)))
}

pub async fn list_for_wallet(
    State(state): State<AppState>,
    Path(wallet): Path<String>,
) -> AppResult<Json<Vec<PublicNode>>> {
    auth::decode_solana_pubkey(&wallet).map_err(AppError::from)?;
    let nodes = state.store().list_nodes_by_wallet(&wallet).await?;
    Ok(Json(nodes.iter().map(PublicNode::from).collect()))
}

#[derive(Debug, Deserialize)]
pub struct ExplorerListQuery {
    #[serde(default = "default_explorer_limit")]
    pub limit: i64,
}

fn default_explorer_limit() -> i64 {
    100
}

pub async fn list_active(
    State(state): State<AppState>,
    Query(q): Query<ExplorerListQuery>,
) -> AppResult<Json<Vec<PublicNode>>> {
    let limit = q.limit.clamp(1, 500);
    let nodes = state
        .store()
        .list_active_nodes(state.config().network.as_str(), limit)
        .await?;
    Ok(Json(nodes.iter().map(PublicNode::from).collect()))
}

#[derive(Debug, Deserialize)]
pub struct ProofsQuery {
    #[serde(default = "default_proof_limit")]
    pub limit: i64,
}

fn default_proof_limit() -> i64 {
    100
}

pub async fn list_proofs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ProofsQuery>,
) -> AppResult<Json<Vec<Proof>>> {
    // Ensure the node exists so callers get 404 vs an empty list ambiguity.
    state.store().get_node(id).await?.ok_or(AppError::NotFound)?;
    let limit = q.limit.clamp(1, 500);
    let proofs = state.store().list_proofs_by_node(id, limit).await?;
    Ok(Json(proofs))
}

#[derive(Debug, Deserialize)]
pub struct SeriesQuery {
    #[serde(default = "default_series_days")]
    pub days: i64,
}

fn default_series_days() -> i64 {
    14
}

pub async fn daily_series(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<SeriesQuery>,
) -> AppResult<Json<Vec<NodeDailyBucket>>> {
    state.store().get_node(id).await?.ok_or(AppError::NotFound)?;
    let series = state.store().node_daily_series(id, q.days).await?;
    Ok(Json(series))
}

fn validate_rpc_endpoint(endpoint: &str) -> AppResult<()> {
    let url = url::Url::parse(endpoint)
        .map_err(|e| AppError::bad_request(format!("invalid rpc_endpoint url: {e}")))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(AppError::bad_request(format!("rpc_endpoint scheme must be http/https, got {other}"))),
    }
    let host = url
        .host_str()
        .ok_or_else(|| AppError::bad_request("rpc_endpoint missing host"))?;
    // Reject localhost / private / link-local hosts — the server can't reach
    // those from Fly, so operators who set them are either confused or trying
    // to bypass the Exposed-RPC verification path.
    if is_unreachable_host(host) {
        return Err(AppError::bad_request(format!(
            "rpc_endpoint host '{host}' is localhost or a private/link-local address — exposed-rpc URLs must be publicly reachable"
        )));
    }
    Ok(())
}

fn is_unreachable_host(host: &str) -> bool {
    let h = host.to_lowercase();
    if matches!(h.as_str(), "localhost" | "ip6-localhost" | "ip6-loopback") {
        return true;
    }
    // Try to parse as IP literal (strip [..] for IPv6).
    let parsed = h
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse::<std::net::IpAddr>();
    if let Ok(ip) = parsed {
        match ip {
            std::net::IpAddr::V4(v4) => {
                return v4.is_loopback()
                    || v4.is_private()
                    || v4.is_link_local()
                    || v4.is_unspecified()
                    || v4.is_broadcast();
            }
            std::net::IpAddr::V6(v6) => {
                return v6.is_loopback() || v6.is_unspecified() || v6.segments()[0] & 0xfe00 == 0xfc00;
            }
        }
    }
    false
}
