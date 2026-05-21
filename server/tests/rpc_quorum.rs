// Integration tests for the trusted-RPC quorum. We spin up small axum servers
// on ephemeral ports that emulate the Zcash JSON-RPC endpoints, then exercise
// the quorum client against them.

use axum::{routing::post, Json, Router};
use depinzcash_server::rpc::{RpcError, ZcashRpcQuorum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};

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

#[derive(Clone)]
enum MockBehavior {
    // Returns a fixed result for any method.
    Fixed(Value),
    // Returns the value at `method`-keyed lookup; falls back to a default.
    PerMethod {
        responses: std::collections::HashMap<String, Value>,
        fallback: Value,
    },
    // Always 500s.
    Error,
}

struct MockServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl MockServer {
    async fn start(behavior: MockBehavior) -> Self {
        let state = Arc::new(Mutex::new(behavior));
        let app = Router::new()
            .route(
                "/",
                post(move |Json(req): Json<JsonRpcReq>| {
                    let state = state.clone();
                    async move {
                        let b = state.lock().await.clone();
                        match b {
                            MockBehavior::Fixed(v) => Ok(Json(JsonRpcResp {
                                jsonrpc: "2.0",
                                id: 1,
                                result: v,
                            })),
                            MockBehavior::PerMethod { responses, fallback } => {
                                let v = responses.get(&req.method).cloned().unwrap_or(fallback);
                                Ok(Json(JsonRpcResp {
                                    jsonrpc: "2.0",
                                    id: 1,
                                    result: v,
                                }))
                            }
                            MockBehavior::Error => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                        }
                    }
                }),
            );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app.into_make_service()).await.unwrap();
        });
        Self { addr, handle }
    }

    fn url(&self) -> String {
        format!("http://{}/", self.addr)
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn quorum_for(servers: &[&MockServer]) -> ZcashRpcQuorum {
    let urls = servers.iter().map(|s| s.url()).collect();
    ZcashRpcQuorum::new(urls, Duration::from_secs(2))
}

// ---- happy paths ---------------------------------------------------------

#[tokio::test]
async fn quorum_agreement_returns_majority() {
    let s1 = MockServer::start(MockBehavior::Fixed(json!(1_234_567))).await;
    let s2 = MockServer::start(MockBehavior::Fixed(json!(1_234_567))).await;
    let s3 = MockServer::start(MockBehavior::Fixed(json!(1_234_567))).await;
    let q = quorum_for(&[&s1, &s2, &s3]);
    let height = q.get_block_count().await.unwrap();
    assert_eq!(height, 1_234_567);
}

#[tokio::test]
async fn quorum_two_of_three_wins() {
    let s1 = MockServer::start(MockBehavior::Fixed(json!(100))).await;
    let s2 = MockServer::start(MockBehavior::Fixed(json!(100))).await;
    let s3 = MockServer::start(MockBehavior::Fixed(json!(999))).await;
    let q = quorum_for(&[&s1, &s2, &s3]);
    let height = q.get_block_count().await.unwrap();
    assert_eq!(height, 100);
}

#[tokio::test]
async fn quorum_handles_one_endpoint_only() {
    let s1 = MockServer::start(MockBehavior::Fixed(json!(7))).await;
    let q = quorum_for(&[&s1]);
    let height = q.get_block_count().await.unwrap();
    assert_eq!(height, 7);
}

#[tokio::test]
async fn block_hash_quorum_returns_string() {
    let s1 = MockServer::start(MockBehavior::Fixed(json!("0000000000abcdef"))).await;
    let s2 = MockServer::start(MockBehavior::Fixed(json!("0000000000abcdef"))).await;
    let q = quorum_for(&[&s1, &s2]);
    let hash = q.get_block_hash(2_500_000).await.unwrap();
    assert_eq!(hash, "0000000000abcdef");
}

#[tokio::test]
async fn per_method_routing() {
    let mut map = std::collections::HashMap::new();
    map.insert("getblockcount".into(), json!(42));
    map.insert("getbestblockhash".into(), json!("aabbcc"));
    let s = MockServer::start(MockBehavior::PerMethod {
        responses: map,
        fallback: json!(null),
    })
    .await;
    let q = ZcashRpcQuorum::new(vec![s.url()], Duration::from_secs(2));
    assert_eq!(q.get_block_count().await.unwrap(), 42);
    assert_eq!(q.get_best_block_hash().await.unwrap(), "aabbcc");
}

// ---- failure paths -------------------------------------------------------

#[tokio::test]
async fn empty_quorum_returns_no_endpoints() {
    let q = ZcashRpcQuorum::new(vec![], Duration::from_secs(1));
    let r = q.get_block_count().await;
    assert!(matches!(r, Err(RpcError::NoEndpoints)));
}

#[tokio::test]
async fn all_endpoints_failing_returns_all_failed() {
    let s1 = MockServer::start(MockBehavior::Error).await;
    let s2 = MockServer::start(MockBehavior::Error).await;
    let q = quorum_for(&[&s1, &s2]);
    let r = q.get_block_count().await;
    assert!(matches!(r, Err(RpcError::AllFailed)), "got: {:?}", r);
}

#[tokio::test]
async fn no_majority_returns_no_quorum() {
    // Three different answers from three nodes → no majority of two.
    let s1 = MockServer::start(MockBehavior::Fixed(json!(1))).await;
    let s2 = MockServer::start(MockBehavior::Fixed(json!(2))).await;
    let s3 = MockServer::start(MockBehavior::Fixed(json!(3))).await;
    let q = quorum_for(&[&s1, &s2, &s3]);
    let r = q.get_block_count().await;
    assert!(matches!(r, Err(RpcError::NoQuorum)), "got: {:?}", r);
}

#[tokio::test]
async fn timeout_treated_as_failure_not_majority() {
    // One responsive node, one slow node → still get the one good answer (1-of-1 majority of
    // the responsive subset). Our quorum logic counts the failing endpoint as a failed
    // response, then checks majority over the full endpoint count — so 1 of 2 isn't a
    // majority. Confirm we get NoQuorum.
    let s1 = MockServer::start(MockBehavior::Fixed(json!(50))).await;
    let s2 = MockServer::start(MockBehavior::Error).await;
    let q = quorum_for(&[&s1, &s2]);
    let r = q.get_block_count().await;
    // 1 of 2 means majority threshold (2/2 + 1 = 2) NOT met.
    assert!(matches!(r, Err(RpcError::NoQuorum)), "got: {:?}", r);
}

#[tokio::test]
async fn is_configured_reports_correctly() {
    let q_empty = ZcashRpcQuorum::new(vec![], Duration::from_secs(1));
    let q_full = ZcashRpcQuorum::new(vec!["http://x".into()], Duration::from_secs(1));
    assert!(!q_empty.is_configured());
    assert!(q_full.is_configured());
}

#[tokio::test]
async fn type_mismatch_returns_other_error() {
    // get_block_count expects a u64. If the RPC returns a string, the parse should fail.
    let s1 = MockServer::start(MockBehavior::Fixed(json!("not a number"))).await;
    let q = quorum_for(&[&s1]);
    let r = q.get_block_count().await;
    assert!(matches!(r, Err(RpcError::Other(_))), "got: {:?}", r);
}
