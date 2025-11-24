use depinzcash_prover::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_proof_generation_basic() {
    // Create a mock node metrics
    let metrics = NodeMetrics {
        block_height: 2450000,
        sync_percentage: 100.0,
        network: "testnet".to_string(),
        uptime_hours: 24.0,
        peer_count: 10,
        blocks_served: 1000,
        zebra_version: "1.5.0".to_string(),
        zebra_binary_hash: "abc123".to_string(),
        timestamp: 1700000000,
    };

    // Create a test config
    let config = Config {
        solana_wallet: "TestSolanaWallet123".to_string(),
        zcash_address: "t1TestAddress".to_string(),
        node_id: Some("test-node".to_string()),
        auto_submit: false,
        api_endpoint: "https://test.local".to_string(),
        api_key: None,
    };

    let proof_gen = ProofGenerator::new(config);

    // This should work even with mock proofs
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(proof_gen.generate_proof(&metrics));

    assert!(result.is_ok());
    let proof = result.unwrap();

    assert_eq!(proof.version, "1.0");
    assert_eq!(proof.node_info.network, "testnet");
    assert_eq!(proof.metrics.block_height, 2450000);
}

#[test]
fn test_reward_calculation() {
    let proof = Proof {
        version: "1.0".to_string(),
        timestamp: 1700000000,
        node_info: NodeInfo {
            zebra_version: "1.5.0".to_string(),
            zebra_binary_hash: "abc123".to_string(),
            network: "mainnet".to_string(),
            node_id: None,
        },
        metrics: ProofMetrics {
            block_height: 2450000,
            sync_percentage: 100.0,
            uptime_hours: 720.0,
            peer_count: 15,
            blocks_served: 5000,
        },
        halo2_proof: "test_proof".to_string(),
        public_inputs: vec!["2450000".to_string(), "1700000000".to_string()],
        wallets: Wallets {
            solana: "test_sol".to_string(),
            zcash: "test_zec".to_string(),
        },
        signature: "test_sig".to_string(),
    };

    let reward = proof.calculate_reward();

    // Should get sync bonus + uptime with multiplier
    assert!(reward.sync_bonus > 0.0);
    assert!(reward.uptime_reward > 0.0);
    assert_eq!(reward.multiplier, 1.5); // Has peers
    assert!(reward.total_zec > 1.0); // Should be substantial for 30 days
}

#[test]
fn test_reward_no_peers() {
    let proof = Proof {
        version: "1.0".to_string(),
        timestamp: 1700000000,
        node_info: NodeInfo {
            zebra_version: "1.5.0".to_string(),
            zebra_binary_hash: "abc123".to_string(),
            network: "mainnet".to_string(),
            node_id: None,
        },
        metrics: ProofMetrics {
            block_height: 2450000,
            sync_percentage: 100.0,
            uptime_hours: 24.0,
            peer_count: 0, // No peers
            blocks_served: 0,
        },
        halo2_proof: "test_proof".to_string(),
        public_inputs: vec!["2450000".to_string(), "1700000000".to_string()],
        wallets: Wallets {
            solana: "test_sol".to_string(),
            zcash: "test_zec".to_string(),
        },
        signature: "test_sig".to_string(),
    };

    let reward = proof.calculate_reward();

    assert_eq!(reward.multiplier, 1.0); // No multiplier without peers
}

#[test]
fn test_proof_serialization() {
    let proof = Proof {
        version: "1.0".to_string(),
        timestamp: 1700000000,
        node_info: NodeInfo {
            zebra_version: "1.5.0".to_string(),
            zebra_binary_hash: "abc123".to_string(),
            network: "mainnet".to_string(),
            node_id: Some("test-node".to_string()),
        },
        metrics: ProofMetrics {
            block_height: 2450000,
            sync_percentage: 100.0,
            uptime_hours: 24.0,
            peer_count: 10,
            blocks_served: 1000,
        },
        halo2_proof: "test_proof".to_string(),
        public_inputs: vec!["2450000".to_string(), "1700000000".to_string()],
        wallets: Wallets {
            solana: "test_sol_wallet".to_string(),
            zcash: "t1test_zec_address".to_string(),
        },
        signature: "test_signature".to_string(),
    };

    // Test save and load
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_proof.json");

    proof.save_to_file(&file_path).unwrap();

    let loaded_proof = Proof::load_from_file(&file_path).unwrap();

    assert_eq!(proof.version, loaded_proof.version);
    assert_eq!(proof.metrics.block_height, loaded_proof.metrics.block_height);
    assert_eq!(proof.wallets.solana, loaded_proof.wallets.solana);
}

#[test]
fn test_config_validation() {
    // Valid config
    let valid_config = Config {
        solana_wallet: "ValidSolanaAddress".to_string(),
        zcash_address: "t1ValidZcashAddress".to_string(),
        node_id: None,
        auto_submit: false,
        api_endpoint: "https://api.test.com".to_string(),
        api_key: None,
    };

    assert!(valid_config.validate().is_ok());

    // Invalid - empty wallet
    let invalid_config = Config {
        solana_wallet: "".to_string(),
        zcash_address: "t1ValidZcashAddress".to_string(),
        node_id: None,
        auto_submit: false,
        api_endpoint: "https://api.test.com".to_string(),
        api_key: None,
    };

    assert!(invalid_config.validate().is_err());

    // Invalid - bad zcash address
    let invalid_zcash = Config {
        solana_wallet: "ValidSolanaAddress".to_string(),
        zcash_address: "invalid_address".to_string(),
        node_id: None,
        auto_submit: false,
        api_endpoint: "https://api.test.com".to_string(),
        api_key: None,
    };

    assert!(invalid_zcash.validate().is_err());
}

#[test]
fn test_partial_sync_rewards() {
    // 90% sync
    let proof_90 = Proof {
        version: "1.0".to_string(),
        timestamp: 1700000000,
        node_info: NodeInfo {
            zebra_version: "1.5.0".to_string(),
            zebra_binary_hash: "abc123".to_string(),
            network: "mainnet".to_string(),
            node_id: None,
        },
        metrics: ProofMetrics {
            block_height: 2200000,
            sync_percentage: 90.0,
            uptime_hours: 1.0,
            peer_count: 0,
            blocks_served: 0,
        },
        halo2_proof: "test".to_string(),
        public_inputs: vec![],
        wallets: Wallets {
            solana: "test".to_string(),
            zcash: "t1test".to_string(),
        },
        signature: "sig".to_string(),
    };

    let reward_90 = proof_90.calculate_reward();
    assert!(reward_90.sync_bonus > 0.0);
    assert!(reward_90.sync_bonus < 0.5); // Less than full sync

    // 75% sync
    let mut proof_75 = proof_90.clone();
    proof_75.metrics.sync_percentage = 75.0;
    let reward_75 = proof_75.calculate_reward();

    assert!(reward_75.sync_bonus < reward_90.sync_bonus);
}
