// Live integration tests for exposed-rpc mode. These talk to a real Zcash
// JSON-RPC endpoint — no mocks. They're #[ignore]'d so `cargo test` doesn't
// flake on third-party uptime; opt in by setting the env var and passing
// --include-ignored.
//
// Run with one of:
//
//   LIVE_ZEBRA_RPC=https://my-zebra.example.com \
//     cargo test --test exposed_rpc_live -- --include-ignored --nocapture
//
//   # or use the same URL for both operator and trusted quorum (smoke-tests the
//   # request/response shape without testing disagreement detection):
//   LIVE_ZEBRA_RPC=https://zebra.example.com \
//   LIVE_TRUSTED_RPC=https://zebra.example.com \
//     cargo test --test exposed_rpc_live -- --include-ignored --nocapture
//
// What each test asserts is at the top of each function. Skipped (printed and
// returned ok) when the relevant env var is unset.

use chrono::Utc;
use depinzcash_server::{
    config::{Config, ZcashNetwork},
    rpc::ZcashRpcQuorum,
    scheduler,
    state::AppState,
    store::SqliteStore,
    types::{Node, NodeKind, NodeStatus, ProofVerdict},
};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

const ENV_OPERATOR: &str = "LIVE_ZEBRA_RPC";
const ENV_TRUSTED: &str = "LIVE_TRUSTED_RPC";

fn operator_url() -> Option<String> {
    std::env::var(ENV_OPERATOR).ok().filter(|s| !s.is_empty())
}

fn trusted_url() -> Option<String> {
    std::env::var(ENV_TRUSTED)
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(operator_url)
}

fn cfg(trusted_rpcs: Vec<String>) -> Config {
    Config {
        bind_addr: "127.0.0.1:0".into(),
        database_url: "sqlite::memory:".into(),
        trusted_rpcs,
        rpc_timeout: Duration::from_secs(15),
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
        max_nodes_per_wallet: 5,
        spl_mint: None,
        solana_cluster: "devnet".into(),
        network: ZcashNetwork::Mainnet,
    }
}

async fn build_state(trusted_rpcs: Vec<String>) -> AppState {
    let store = SqliteStore::connect("sqlite::memory:").await.unwrap();
    store.migrate().await.unwrap();
    let rpc = ZcashRpcQuorum::new(trusted_rpcs.clone(), Duration::from_secs(15));
    AppState::new(cfg(trusted_rpcs), store, rpc)
}

fn make_node(rpc_endpoint: &str) -> Node {
    Node {
        id: Uuid::new_v4(),
        wallet: "WalletLIVE".into(),
        kind: NodeKind::ZebraFull,
        label: Some("live-test".into()),
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

/// Returns true if `s` looks like a Zcash mainnet block hash:
/// 64-char lowercase hex, starts with several leading zeros (mainnet difficulty).
fn looks_like_zcash_hash(s: &str) -> bool {
    let s = s.trim().trim_start_matches("0x");
    if s.len() != 64 {
        return false;
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    // Mainnet block hashes always have several leading zero hex chars — pow target.
    s.starts_with("00000000")
}

// ---- tests ----------------------------------------------------------------

#[tokio::test]
#[ignore = "needs LIVE_ZEBRA_RPC env var pointing at a real Zcash JSON-RPC endpoint"]
async fn live_getblockcount_returns_plausible_height() {
    let Some(op) = operator_url() else {
        eprintln!("SKIP: {ENV_OPERATOR} not set");
        return;
    };

    // Use the same URL as the quorum so we just smoke-test connectivity.
    let rpc = ZcashRpcQuorum::new(vec![op.clone()], Duration::from_secs(15));
    let v = rpc
        .call_single(&op, "getblockcount", json!([]))
        .await
        .expect("getblockcount call should succeed");
    let height = v
        .as_u64()
        .unwrap_or_else(|| panic!("expected u64 height, got {v}"));

    eprintln!("live getblockcount → {height}");
    // Zcash mainnet was already past block 3,000,000 well before this code shipped.
    assert!(height > 3_000_000, "height {height} looks too small for mainnet");
}

#[tokio::test]
#[ignore = "needs LIVE_ZEBRA_RPC env var pointing at a real Zcash JSON-RPC endpoint"]
async fn live_getblockhash_returns_real_zcash_mainnet_hash() {
    let Some(op) = operator_url() else {
        eprintln!("SKIP: {ENV_OPERATOR} not set");
        return;
    };

    let rpc = ZcashRpcQuorum::new(vec![op.clone()], Duration::from_secs(15));
    let count = rpc
        .call_single(&op, "getblockcount", json!([]))
        .await
        .unwrap()
        .as_u64()
        .unwrap();

    // Ask for a height a few blocks back — guarantees it's not still being
    // reorganised, and matches what poll_one_node does in practice.
    let target = count.saturating_sub(5);
    let v = rpc
        .call_single(&op, "getblockhash", json!([target]))
        .await
        .expect("getblockhash call should succeed");
    let hash = v.as_str().expect("expected string hash").to_string();

    eprintln!("live getblockhash({target}) → {hash}");
    assert!(
        looks_like_zcash_hash(&hash),
        "hash {hash} doesn't look like a mainnet zcash block hash (64 hex chars, leading zeros)"
    );
}

#[tokio::test]
#[ignore = "needs LIVE_ZEBRA_RPC + (optional) LIVE_TRUSTED_RPC env vars"]
async fn live_poll_one_node_credits_against_real_quorum() {
    let Some(op) = operator_url() else {
        eprintln!("SKIP: {ENV_OPERATOR} not set");
        return;
    };
    let trusted = trusted_url().expect("trusted url derived above");

    let state = build_state(vec![trusted.clone()]).await;
    let node = make_node(&op);
    state
        .store()
        .insert_node(&node, "live-auth-token")
        .await
        .unwrap();

    // Refresh the trusted tip the same way the scheduler does.
    let tip = state
        .rpc()
        .get_block_count()
        .await
        .expect("trusted quorum should answer getblockcount");
    state.set_trusted_tip(tip).await;
    eprintln!("live trusted tip → {tip}");

    scheduler::poll_one_node(&state, &node, Some(tip))
        .await
        .expect("poll_one_node should succeed end-to-end against live RPC");

    let proofs = state.store().list_proofs_by_node(node.id, 5).await.unwrap();
    assert_eq!(proofs.len(), 1, "exactly one proof should have been written");

    let p = &proofs[0];
    eprintln!(
        "live proof: verdict={} height={} hash={} points={}",
        p.verdict.as_str(),
        p.claimed_height,
        p.claimed_block_hash,
        p.points_awarded
    );

    // Whether accepted or rejected, the block hash must look real — the
    // operator endpoint returned it.
    assert!(looks_like_zcash_hash(&p.claimed_block_hash));

    if p.verdict == ProofVerdict::Accepted {
        // Same URL on both sides → must agree.
        assert!(p.points_awarded > 0);
        let n = state.store().get_node(node.id).await.unwrap().unwrap();
        assert_eq!(n.status, NodeStatus::Active);
        assert!(n.points > 0);
    } else {
        // If the operator URL was different from the trusted URL and they
        // disagreed at the same height, we expect a rejection. The reject
        // reason is human-readable.
        assert!(p.reject_reason.is_some());
        eprintln!("  rejected: {}", p.reject_reason.as_deref().unwrap_or(""));
    }
}
