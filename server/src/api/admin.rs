use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{error::AppError, merkle, state::AppState, types::NodeStatus};

#[derive(Debug, Serialize)]
pub struct PublishSnapshotResponse {
    pub cycle: i64,
    pub merkle_root: String,
    pub leaves: usize,
    pub total_points: u64,
}

pub async fn publish_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PublishSnapshotResponse>, AppError> {
    require_admin(&state, &headers)?;

    let resp = merkle::publish_snapshot(&state).await?;
    Ok(Json(PublishSnapshotResponse {
        cycle: resp.cycle,
        merkle_root: resp.merkle_root,
        leaves: resp.leaves,
        total_points: resp.total_points,
    }))
}

// DELETE the node, all its proofs (CASCADE), all its challenges (CASCADE),
// and zero its points contribution. Use this to remove farmers / fake nodes.
pub async fn purge_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    require_admin(&state, &headers)?;
    let pool = state.store().pool();
    let res = sqlx::query("DELETE FROM nodes WHERE id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("purge_node: {e}")))?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tracing::warn!(node_id = %id, "node purged by admin");
    Ok(Json(json!({ "purged": id.to_string(), "ok": true })))
}

// Suspend (but don't delete) — keeps the row for audit, stops rewards.
pub async fn suspend_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    require_admin(&state, &headers)?;
    state
        .store()
        .update_node_status(id, NodeStatus::Suspended)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("suspend_node: {e}")))?;
    tracing::warn!(node_id = %id, "node suspended by admin");
    Ok(Json(json!({ "suspended": id.to_string(), "ok": true })))
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let configured = state
        .config()
        .admin_api_key
        .as_deref()
        .ok_or(AppError::Forbidden)?;
    let provided = headers
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    if !constant_time_eq(configured.as_bytes(), provided.as_bytes()) {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

