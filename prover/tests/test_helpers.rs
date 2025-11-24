// Test helpers and mock data generators

use depinzcash_prover::*;

/// Create a mock NodeMetrics for testing
pub fn mock_node_metrics() -> NodeMetrics {
    NodeMetrics {
        block_height: 2450123,
        sync_percentage: 100.0,
        network: "testnet".to_string(),
        uptime_hours: 168.0, // 7 days
        peer_count: 15,
        blocks_served: 5000,
        zebra_version: "1.5.0".to_string(),
        zebra_binary_hash: "test_hash_abc123".to_string(),
        timestamp: 1700000000,
    }
}

/// Create a mock Config for testing
pub fn mock_config() -> Config {
    Config {
        solana_wallet: "TestSolWallet123456789".to_string(),
        zcash_address: "t1TestZcashAddress123".to_string(),
        node_id: Some("test-node-001".to_string()),
        auto_submit: false,
        api_endpoint: "https://test.depinzcash.io/api/submit".to_string(),
        api_key: None,
    }
}

/// Create a mock Proof for testing
pub fn mock_proof() -> Proof {
    Proof {
        version: "1.0".to_string(),
        timestamp: 1700000000,
        node_info: NodeInfo {
            zebra_version: "1.5.0".to_string(),
            zebra_binary_hash: "test_hash".to_string(),
            network: "testnet".to_string(),
            node_id: Some("test-node".to_string()),
        },
        metrics: ProofMetrics {
            block_height: 2450123,
            sync_percentage: 100.0,
            uptime_hours: 168.0,
            peer_count: 15,
            blocks_served: 5000,
        },
        halo2_proof: "mock_halo2_proof_data".to_string(),
        public_inputs: vec![
            "2450123".to_string(),
            "1700000000".to_string(),
            "testnet".to_string(),
        ],
        wallets: Wallets {
            solana: "TestSolWallet123".to_string(),
            zcash: "t1TestZcash123".to_string(),
        },
        signature: "mock_signature_ed25519".to_string(),
    }
}

/// Create NodeMetrics with custom sync percentage
pub fn mock_metrics_with_sync(sync_pct: f64) -> NodeMetrics {
    let mut metrics = mock_node_metrics();
    metrics.sync_percentage = sync_pct;
    metrics
}

/// Create NodeMetrics with custom peer count
pub fn mock_metrics_with_peers(peer_count: u32) -> NodeMetrics {
    let mut metrics = mock_node_metrics();
    metrics.peer_count = peer_count;
    metrics
}

/// Create NodeMetrics with custom uptime
pub fn mock_metrics_with_uptime(hours: f64) -> NodeMetrics {
    let mut metrics = mock_node_metrics();
    metrics.uptime_hours = hours;
    metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_helpers() {
        let metrics = mock_node_metrics();
        assert_eq!(metrics.sync_percentage, 100.0);

        let config = mock_config();
        assert!(!config.solana_wallet.is_empty());

        let proof = mock_proof();
        assert_eq!(proof.version, "1.0");
    }

    #[test]
    fn test_custom_sync() {
        let metrics = mock_metrics_with_sync(85.5);
        assert_eq!(metrics.sync_percentage, 85.5);
    }

    #[test]
    fn test_custom_peers() {
        let metrics = mock_metrics_with_peers(25);
        assert_eq!(metrics.peer_count, 25);
    }
}
