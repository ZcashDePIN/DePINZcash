// Integration tests for the exposed-rpc verification mode.
//
// We stand up two mock Zcash JSON-RPC servers — one acting as the operator's
// publicly-exposed node, one as the trusted quorum — and exercise the
// scheduler's poll_one_node directly. No live network, no flakiness.

use axum::{routing::post, Json, Router};
use chrono::Utc;
use depinzcash_server::{
    config::{Config, ZcashNetwork},
    rpc::ZcashRpcQuorum,
    scheduler,
    state::AppState,
    store::SqliteStore,
    types::{Node, NodeKind, NodeStatus, ProofVerdict},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use uuid::Uuid;

// ---- mock zcashd ----------------------------------------------------------

#[derive(Deserialize)]
struct JsonRpcReq {
    method: String,
    #[serde(default, rename = "params")]
    _params: Value,
}

#[derive(Serialize)]
struct JsonRpcResp {
    jsonrpc: &'static str,
    id: u32,
    result: Value,
}

struct MockNode {
    addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl MockNode {
    async fn start(responses: HashMap<String, Value>) -> Self {
        let state = Arc::new(Mutex::new(responses));
        let app = Router::new().route(
            "/",
            post(move |Json(req): Json<JsonRpcReq>| {
                let state = state.clone();
                async move {
                    let r = state.lock().await;
                    let v = r.get(&req.method).cloned().unwrap_or(Value::Null);
                    Json(JsonRpcResp {
                        jsonrpc: "2.0",
                        id: 1,
                        result: v,
                    })
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        MockNode { addr, handle }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn shutdown(self) {
        self.handle.abort();
    }
}

fn responses(height: u64, hash: &str) -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("getblockcount".into(), json!(height));
    m.insert("getblockhash".into(), json!(hash));
    m.insert("getbestblockhash".into(), json!(hash));
    m
}

// ---- fixture builders -----------------------------------------------------

fn cfg(trusted_rpcs: Vec<String>) -> Config {
    Config {
        bind_addr: "127.0.0.1:0".into(),
        database_url: "sqlite::memory:".into(),
        trusted_rpcs,
        rpc_timeout: Duration::from_secs(2),
        admin_api_key: Some("admin-key".into()),
        cors_allowed_origins: vec![],
        scheduler_enabled: false,
        heartbeat_interval: Duration::from_secs(60),
        challenge_check_interval: Duration::from_secs(60),
        uptime_reward_interval: Duration::from_secs(60),
        snapshot_interval: None,
        exposed_rpc_poll_interval: Some(Duration::from_secs(60)),
        max_height_drift: 8,
        max_clock_skew: Duration::from_secs(15 * 60),
        rate_limit_enabled: false,
        rate_limit_per_second: 1000,
        rate_limit_burst: 5000,
        registration_enabled: true,
        spl_mint: None,
        solana_cluster: "devnet".into(),
        network: ZcashNetwork::Mainnet,
    }
}

async fn build_state(trusted_rpcs: Vec<String>) -> AppState {
    let store = SqliteStore::connect("sqlite::memory:").await.unwrap();
    store.migrate().await.unwrap();
    let rpc = ZcashRpcQuorum::new(trusted_rpcs.clone(), Duration::from_secs(2));
    AppState::new(cfg(trusted_rpcs), store, rpc)
}

fn make_node(rpc_endpoint: &str) -> Node {
    Node {
        id: Uuid::new_v4(),
        wallet: "WalletABC".into(),
        kind: NodeKind::ZebraFull,
        label: Some("test-node".into()),
        rpc_endpoint: Some(rpc_endpoint.to_string()),
        network: "mainnet".into(),
        status: NodeStatus::Registered,
        last_height: None,
        last_block_hash: None,
        last_proof_at: None,
        registered_at: Utc::now(),
        points: 0,
        uptime_seconds: 0,
    }
}

// ---- tests ----------------------------------------------------------------

#[tokio::test]
async fn happy_path_credits_node_when_hash_matches_quorum() {
    let height = 3_350_000u64;
    let hash = "0000000000abcdef1234567890abcdef1234567890abcdef1234567890abcd01";

    let operator = MockNode::start(responses(height, hash)).await;
    let trusted = MockNode::start(responses(height, hash)).await;

    let state = build_state(vec![trusted.url()]).await;
    let node = make_node(&operator.url());
    state.store().insert_node(&node, "auth-token-1").await.unwrap();

    scheduler::poll_one_node(&state, &node, Some(height))
        .await
        .expect("poll should succeed");

    let proofs = state.store().list_proofs_by_node(node.id, 10).await.unwrap();
    assert_eq!(proofs.len(), 1, "exactly one proof inserted");
    let p = &proofs[0];
    assert_eq!(p.verdict, ProofVerdict::Accepted);
    assert_eq!(p.claimed_height, height);
    assert_eq!(p.claimed_block_hash, hash);
    assert_eq!(p.binary_hash.as_deref(), Some("exposed-rpc-poll"));
    assert!(p.points_awarded > 0, "accepted proof must award points");

    // Node row should reflect the acceptance (status -> active, points credited).
    let refreshed = state.store().get_node(node.id).await.unwrap().unwrap();
    assert_eq!(refreshed.status, NodeStatus::Active);
    assert_eq!(refreshed.last_height, Some(height));
    assert_eq!(refreshed.last_block_hash.as_deref(), Some(hash));
    assert!(refreshed.points > 0);

    operator.shutdown();
    trusted.shutdown();
}

#[tokio::test]
async fn hash_mismatch_marks_proof_rejected_and_does_not_credit() {
    let height = 3_350_000u64;
    let operator_hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let trusted_hash = "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe";

    let operator = MockNode::start(responses(height, operator_hash)).await;
    let trusted = MockNode::start(responses(height, trusted_hash)).await;

    let state = build_state(vec![trusted.url()]).await;
    let node = make_node(&operator.url());
    state.store().insert_node(&node, "auth-token-2").await.unwrap();

    scheduler::poll_one_node(&state, &node, Some(height))
        .await
        .expect("poll should succeed even when hashes disagree");

    let proofs = state.store().list_proofs_by_node(node.id, 10).await.unwrap();
    assert_eq!(proofs.len(), 1);
    let p = &proofs[0];
    assert_eq!(p.verdict, ProofVerdict::Rejected);
    assert_eq!(p.points_awarded, 0);
    assert!(p.reject_reason.as_deref().unwrap_or("").contains("exposed-rpc"));

    let refreshed = state.store().get_node(node.id).await.unwrap().unwrap();
    assert_eq!(refreshed.points, 0, "rejected proof must not credit");
    assert_ne!(refreshed.status, NodeStatus::Active);

    operator.shutdown();
    trusted.shutdown();
}

#[tokio::test]
async fn polling_same_tip_twice_is_idempotent() {
    let height = 3_351_000u64;
    let hash = "00000000000000000000000000000000000000000000000000000000abcd1111";

    let operator = MockNode::start(responses(height, hash)).await;
    let trusted = MockNode::start(responses(height, hash)).await;

    let state = build_state(vec![trusted.url()]).await;
    let node = make_node(&operator.url());
    state.store().insert_node(&node, "auth-token-3").await.unwrap();

    // First poll: inserts.
    scheduler::poll_one_node(&state, &node, Some(height))
        .await
        .unwrap();
    let first = state.store().list_proofs_by_node(node.id, 10).await.unwrap();
    assert_eq!(first.len(), 1);
    let credited_once = state.store().get_node(node.id).await.unwrap().unwrap().points;

    // Second poll on the unchanged tip: no new row, no extra points.
    scheduler::poll_one_node(&state, &node, Some(height))
        .await
        .unwrap();
    let second = state.store().list_proofs_by_node(node.id, 10).await.unwrap();
    assert_eq!(second.len(), 1, "same (height, hash) must dedupe via UNIQUE");
    let credited_twice = state.store().get_node(node.id).await.unwrap().unwrap().points;
    assert_eq!(credited_once, credited_twice, "no double credit on idle tip");

    operator.shutdown();
    trusted.shutdown();
}

#[tokio::test]
async fn node_too_far_behind_trusted_tip_is_skipped() {
    let operator_height = 60_000u64; // way behind
    let trusted_height = 3_350_000u64;
    let hash = "0000000077e22a7adb7f5988023932a3401d7b6884181aaa19d5e2d299529b41";

    let operator = MockNode::start(responses(operator_height, hash)).await;
    let trusted = MockNode::start(responses(trusted_height, hash)).await;

    let state = build_state(vec![trusted.url()]).await;
    let node = make_node(&operator.url());
    state.store().insert_node(&node, "auth-token-4").await.unwrap();

    scheduler::poll_one_node(&state, &node, Some(trusted_height))
        .await
        .unwrap();

    let proofs = state.store().list_proofs_by_node(node.id, 10).await.unwrap();
    assert!(proofs.is_empty(), "drift-out node must not produce a proof");
    let refreshed = state.store().get_node(node.id).await.unwrap().unwrap();
    assert_eq!(refreshed.points, 0);

    operator.shutdown();
    trusted.shutdown();
}

#[tokio::test]
async fn node_without_rpc_endpoint_is_no_op() {
    let trusted = MockNode::start(responses(100, "h")).await;
    let state = build_state(vec![trusted.url()]).await;

    let mut node = make_node("http://placeholder.invalid");
    node.rpc_endpoint = None; // simulate registered without exposed url
    state.store().insert_node(&node, "auth-token-5").await.unwrap();

    scheduler::poll_one_node(&state, &node, Some(100))
        .await
        .expect("missing rpc_endpoint is not an error, just a no-op");

    let proofs = state.store().list_proofs_by_node(node.id, 10).await.unwrap();
    assert!(proofs.is_empty());

    trusted.shutdown();
}
