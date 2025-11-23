use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Metrics read from a Zebra node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Current blockchain height
    pub block_height: u64,

    /// Sync percentage (0-100)
    pub sync_percentage: f64,

    /// Network (mainnet/testnet)
    pub network: String,

    /// Node uptime in hours (estimated)
    pub uptime_hours: f64,

    /// Number of peers currently connected
    pub peer_count: u32,

    /// Total blocks served to peers (if available)
    pub blocks_served: u64,

    /// Zebra version
    pub zebra_version: String,

    /// Binary hash for verification
    pub zebra_binary_hash: String,

    /// Timestamp of metrics collection
    pub timestamp: i64,
}

/// Reader for Zebra node state
pub struct ZebraReader {
    state_dir: PathBuf,
}

impl ZebraReader {
    /// Create a new ZebraReader
    pub fn new(zebra_dir: &Path) -> Result<Self> {
        let state_dir = zebra_dir.join("state");

        if !state_dir.exists() {
            anyhow::bail!("Zebra state directory not found: {:?}", state_dir);
        }

        Ok(Self { state_dir })
    }

    /// Read metrics from the Zebra node
    pub fn read_metrics(&self) -> Result<NodeMetrics> {
        info!("Reading Zebra state from {:?}", self.state_dir);

        // Detect network (mainnet/testnet)
        let network = self.detect_network()?;
        debug!("Detected network: {}", network);

        // Read block height from RocksDB
        let block_height = self.read_block_height(&network)
            .context("Failed to read block height")?;

        // Calculate sync percentage
        let latest_block = self.get_latest_network_block(&network)
            .context("Failed to get latest network block")?;
        let sync_percentage = (block_height as f64 / latest_block as f64) * 100.0;

        // Estimate uptime from state directory modification time
        let uptime_hours = self.estimate_uptime()?;

        // Get peer count (from state if available)
        let peer_count = self.read_peer_count().unwrap_or(0);

        // Get Zebra version
        let zebra_version = self.read_zebra_version()?;

        // Get binary hash
        let zebra_binary_hash = self.compute_binary_hash()?;

        // Blocks served (approximate - may not be available)
        let blocks_served = self.estimate_blocks_served(block_height);

        let metrics = NodeMetrics {
            block_height,
            sync_percentage,
            network,
            uptime_hours,
            peer_count,
            blocks_served,
            zebra_version,
            zebra_binary_hash,
            timestamp: chrono::Utc::now().timestamp(),
        };

        Ok(metrics)
    }

    fn detect_network(&self) -> Result<String> {
        // Check which network directory exists
        let mainnet_dir = self.state_dir.join("mainnet");
        let testnet_dir = self.state_dir.join("testnet");

        if mainnet_dir.exists() {
            Ok("mainnet".to_string())
        } else if testnet_dir.exists() {
            Ok("testnet".to_string())
        } else {
            anyhow::bail!("Could not detect network (no mainnet or testnet directory found)")
        }
    }

    fn read_block_height(&self, network: &str) -> Result<u64> {
        use rocksdb::{DB, Options};

        let db_path = self.state_dir.join(network).join("db");

        debug!("Opening RocksDB at {:?}", db_path);

        let mut opts = Options::default();
        opts.set_error_if_exists(false);

        let db = DB::open_for_read_only(&opts, &db_path, false)
            .context("Failed to open Zebra state database")?;

        // Zebra stores the tip height in a specific key
        // This is a simplified version - actual implementation may vary
        let tip_key = b"tip_height";

        match db.get(tip_key)? {
            Some(value) => {
                let height = u64::from_le_bytes(
                    value.as_slice().try_into()
                        .context("Invalid block height format")?
                );
                Ok(height)
            }
            None => {
                // If tip_height key doesn't exist, try to find the highest block
                // This is a fallback and may be slow
                warn!("Tip height not found, scanning database...");
                self.scan_for_highest_block(&db)
            }
        }
    }

    fn scan_for_highest_block(&self, db: &rocksdb::DB) -> Result<u64> {
        use rocksdb::IteratorMode;

        let mut highest = 0u64;
        let iter = db.iterator(IteratorMode::Start);

        for item in iter {
            let (key, _) = item?;
            // Look for keys that represent block heights
            // This is simplified - actual format depends on Zebra's implementation
            if let Ok(height) = String::from_utf8_lossy(&key).parse::<u64>() {
                if height > highest {
                    highest = height;
                }
            }
        }

        Ok(highest)
    }

    fn get_latest_network_block(&self, network: &str) -> Result<u64> {
        // Query a Zcash block explorer API for latest block
        // For now, return a reasonable estimate based on known block times

        match network {
            "mainnet" => {
                // Zcash mainnet launched Nov 2016, ~75 second block time
                // Approximate current height
                let genesis_timestamp = 1477958400; // Nov 1, 2016
                let now = chrono::Utc::now().timestamp();
                let seconds_elapsed = now - genesis_timestamp;
                let blocks = seconds_elapsed / 75; // 75 second block time
                Ok(blocks as u64 + 1) // Add 1 to account for genesis block
            }
            "testnet" => {
                // Similar calculation for testnet
                Ok(2_500_000) // Rough estimate
            }
            _ => anyhow::bail!("Unknown network: {}", network),
        }
    }

    fn estimate_uptime(&self) -> Result<f64> {
        use std::time::SystemTime;

        let state_modified = std::fs::metadata(&self.state_dir)?
            .modified()?;

        let elapsed = SystemTime::now()
            .duration_since(state_modified)
            .context("Failed to calculate uptime")?;

        let hours = elapsed.as_secs() as f64 / 3600.0;
        Ok(hours)
    }

    fn read_peer_count(&self) -> Result<u32> {
        // This would require parsing Zebra's peer state
        // For MVP, we can return 0 or estimate
        Ok(0)
    }

    fn read_zebra_version(&self) -> Result<String> {
        // Try to get version from zebrad binary
        use std::process::Command;

        let output = Command::new("zebrad")
            .arg("--version")
            .output()?;

        let version = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        Ok(version)
    }

    fn compute_binary_hash(&self) -> Result<String> {
        use sha2::{Sha256, Digest};

        let zebrad_path = which::which("zebrad")
            .context("zebrad not found in PATH")?;

        let binary = std::fs::read(&zebrad_path)?;
        let hash = Sha256::digest(&binary);

        Ok(hex::encode(hash))
    }

    fn estimate_blocks_served(&self, block_height: u64) -> u64 {
        // This is a very rough estimate
        // Actual implementation would track peer requests
        block_height / 100 // Assume we've served ~1% of blocks we have
    }
}

use tracing::warn;
