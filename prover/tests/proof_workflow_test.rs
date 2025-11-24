// End-to-end proof workflow tests

use depinzcash_prover::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_full_proof_workflow() {
    // Setup test data
    let config = Config {
        solana_wallet: "TestWallet123".to_string(),
        zcash_address: "t1TestAddr".to_string(),
        node_id: Some("workflow-test".to_string()),
        auto_submit: false,
        api_endpoint: "https://test.local".to_string(),
        api_key: None,
    };

    let metrics = NodeMetrics {
        block_height: 2450000,
        sync_percentage: 100.0,
        network: "testnet".to_string(),
        uptime_hours: 72.0, // 3 days
        peer_count: 20,
        blocks_served: 3000,
        zebra_version: "1.5.0".to_string(),
        zebra_binary_hash: "test_binary_hash".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Generate proof
    let proof_gen = ProofGenerator::new(config.clone());
    let proof_result = proof_gen.generate_proof(&metrics).await;

    assert!(proof_result.is_ok(), "Proof generation failed");

    let proof = proof_result.unwrap();

    // Verify proof structure
    assert_eq!(proof.version, "1.0");
    assert_eq!(proof.node_info.network, "testnet");
    assert_eq!(proof.metrics.block_height, 2450000);
    assert_eq!(proof.wallets.solana, config.solana_wallet);
    assert!(!proof.halo2_proof.is_empty());
    assert!(!proof.signature.is_empty());

    // Test serialization
    let temp_dir = tempdir().unwrap();
    let proof_path = temp_dir.path().join("test_proof.json");

    proof.save_to_file(&proof_path).unwrap();

    // Verify file was created
    assert!(proof_path.exists());

    // Load it back
    let loaded_proof = Proof::load_from_file(&proof_path).unwrap();
    assert_eq!(proof.metrics.block_height, loaded_proof.metrics.block_height);
}

#[test]
fn test_reward_scenarios() {
    // Scenario 1: Full sync, serving peers, 30 days
    let proof1 = create_test_proof(100.0, 720.0, 15);
    let reward1 = proof1.calculate_reward();

    println!("Scenario 1 - Full sync, 30 days, serving peers:");
    println!("  Sync bonus: {}", reward1.sync_bonus);
    println!("  Uptime reward: {}", reward1.uptime_reward);
    println!("  Multiplier: {}x", reward1.multiplier);
    println!("  Total: {} ZEC\n", reward1.total_zec);

    assert!(reward1.total_zec > 1.0);
    assert_eq!(reward1.multiplier, 1.5);

    // Scenario 2: Partial sync, no peers, 1 week
    let proof2 = create_test_proof(85.0, 168.0, 0);
    let reward2 = proof2.calculate_reward();

    println!("Scenario 2 - 85% sync, 1 week, no peers:");
    println!("  Sync bonus: {}", reward2.sync_bonus);
    println!("  Uptime reward: {}", reward2.uptime_reward);
    println!("  Multiplier: {}x", reward2.multiplier);
    println!("  Total: {} ZEC\n", reward2.total_zec);

    assert!(reward2.total_zec < reward1.total_zec);
    assert_eq!(reward2.multiplier, 1.0);

    // Scenario 3: Full sync, serving many peers, 14 days
    let proof3 = create_test_proof(100.0, 336.0, 50);
    let reward3 = proof3.calculate_reward();

    println!("Scenario 3 - Full sync, 14 days, 50 peers:");
    println!("  Sync bonus: {}", reward3.sync_bonus);
    println!("  Uptime reward: {}", reward3.uptime_reward);
    println!("  Multiplier: {}x", reward3.multiplier);
    println!("  Total: {} ZEC\n", reward3.total_zec);

    assert_eq!(reward3.sync_bonus, 0.5);
    assert_eq!(reward3.multiplier, 1.5);
}

#[test]
fn test_sync_tiers() {
    let tests = vec![
        (100.0, 0.5),   // Full sync
        (99.0, 0.375),  // 90-99%
        (90.0, 0.375),  // 90-99%
        (89.0, 0.25),   // 75-89%
        (75.0, 0.25),   // 75-89%
        (74.0, 0.0),    // Below threshold
    ];

    for (sync_pct, expected_bonus) in tests {
        let proof = create_test_proof(sync_pct, 1.0, 0);
        let reward = proof.calculate_reward();

        assert_eq!(
            reward.sync_bonus,
            expected_bonus,
            "Sync {}% should give {} ZEC bonus",
            sync_pct,
            expected_bonus
        );
    }
}

#[test]
fn test_peer_multiplier() {
    // With peers
    let with_peers = create_test_proof(100.0, 100.0, 10);
    let reward_with = with_peers.calculate_reward();

    // Without peers
    let without_peers = create_test_proof(100.0, 100.0, 0);
    let reward_without = without_peers.calculate_reward();

    assert_eq!(reward_with.multiplier, 1.5);
    assert_eq!(reward_without.multiplier, 1.0);
    assert!(reward_with.total_zec > reward_without.total_zec);
}

// Helper function
fn create_test_proof(sync_pct: f64, uptime_hours: f64, peer_count: u32) -> Proof {
    Proof {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        node_info: NodeInfo {
            zebra_version: "1.5.0".to_string(),
            zebra_binary_hash: "test".to_string(),
            network: "testnet".to_string(),
            node_id: None,
        },
        metrics: ProofMetrics {
            block_height: 2450000,
            sync_percentage: sync_pct,
            uptime_hours,
            peer_count,
            blocks_served: 1000,
        },
        halo2_proof: "test".to_string(),
        public_inputs: vec![],
        wallets: Wallets {
            solana: "test_sol".to_string(),
            zcash: "t1test_zec".to_string(),
        },
        signature: "sig".to_string(),
    }
}
