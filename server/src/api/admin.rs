use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
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

// Mass cleanup: delete fake / spam nodes in two passes:
//   1. Delete every node with last_height < min_real_height (fake block heights)
//   2. For each wallet with > max_nodes_per_wallet, keep the oldest N and delete the rest
//
// Default is dry_run=true: returns what WOULD be deleted without touching the DB.
// Pass ?confirm=true to actually run it.
#[derive(Debug, Deserialize)]
pub struct CleanupQuery {
    #[serde(default)]
    pub confirm: bool,
    #[serde(default = "default_batch")]
    pub batch: i64,
}

fn default_batch() -> i64 {
    500
}

pub async fn cleanup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<CleanupQuery>,
) -> Result<Json<Value>, AppError> {
    require_admin(&state, &headers)?;

    let pool = state.store().pool();
    let min_height = state.config().min_real_height as i64;
    let max_per_wallet = state.config().max_nodes_per_wallet as i64;

    // Pass 1: count nodes with fake heights (last_height < min_real_height OR
    // last_height IS NULL but they have a last_proof_at, meaning they submitted
    // something but at a garbage height).
    let fake_height_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(1) FROM nodes
           WHERE last_height IS NOT NULL AND last_height < ?1 AND last_height > 0"#,
    )
    .bind(min_height)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("cleanup count fake: {e}")))?;

    // Pass 2: count excess nodes per wallet (beyond max_per_wallet).
    // ROW_NUMBER over (PARTITION BY wallet ORDER BY registered_at ASC) — keep
    // the first N, mark the rest for deletion.
    let excess_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(1) FROM (
             SELECT id, ROW_NUMBER() OVER (PARTITION BY wallet ORDER BY registered_at ASC) AS rn
             FROM nodes
           ) WHERE rn > ?1"#,
    )
    .bind(max_per_wallet)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("cleanup count excess: {e}")))?;

    if !q.confirm {
        return Ok(Json(json!({
            "dry_run": true,
            "would_delete_fake_height": fake_height_count,
            "would_delete_excess_per_wallet": excess_count,
            "min_real_height": min_height,
            "max_nodes_per_wallet": max_per_wallet,
            "batch_size": q.batch,
            "hint": "add ?confirm=true to execute (deletes batch_size per call, repeat until 0)"
        })));
    }

    // Batched deletes — keeps the DB lock short so other requests don't 503.
    // Call repeatedly until both counts return 0.
    let batch = q.batch.clamp(50, 2000);

    // Pass 1: delete a batch of fake-height nodes.
    let deleted_fake = sqlx::query(
        r#"DELETE FROM nodes WHERE id IN (
             SELECT id FROM nodes
             WHERE last_height IS NOT NULL AND last_height < ?1 AND last_height > 0
             LIMIT ?2
           )"#,
    )
    .bind(min_height)
    .bind(batch)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("cleanup delete fake: {e}")))?
    .rows_affected();

    // Pass 2: delete a batch of excess-per-wallet nodes.
    let deleted_excess = sqlx::query(
        r#"DELETE FROM nodes WHERE id IN (
             SELECT id FROM (
               SELECT id, ROW_NUMBER() OVER (PARTITION BY wallet ORDER BY registered_at ASC) AS rn
               FROM nodes
             ) WHERE rn > ?1
             LIMIT ?2
           )"#,
    )
    .bind(max_per_wallet)
    .bind(batch)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("cleanup delete excess: {e}")))?
    .rows_affected();

    let total = deleted_fake + deleted_excess;
    tracing::warn!(
        deleted_fake,
        deleted_excess,
        total,
        batch,
        "admin cleanup batch executed"
    );

    Ok(Json(json!({
        "dry_run": false,
        "deleted_fake_height": deleted_fake,
        "deleted_excess_per_wallet": deleted_excess,
        "total_deleted": total,
        "remaining_fake_height": fake_height_count - deleted_fake as i64,
        "remaining_excess": excess_count - deleted_excess as i64,
        "ok": true
    })))
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

