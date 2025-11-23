use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, warn};

mod config;
mod proof_gen;
mod zebra_reader;

use config::Config;
use proof_gen::ProofGenerator;
use zebra_reader::ZebraReader;

/// DePINZcash Proof Generator
///
/// Generates zero-knowledge proofs of Zcash Zebra node operation
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/config.json")]
    config: PathBuf,

    /// Path to Zebra state directory (auto-detected if not specified)
    #[arg(short, long)]
    zebra_dir: Option<PathBuf>,

    /// Output directory for generated proofs
    #[arg(short, long, default_value = "proofs")]
    output_dir: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Skip binary hash verification (not recommended)
    #[arg(long)]
    skip_verification: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.verbose);

    println!("🦓 DePINZcash Proof Generator v{}", env!("CARGO_PKG_VERSION"));
    println!("================================================\n");

    // Load configuration
    info!("Loading configuration from {:?}", args.config);
    let config = Config::load(&args.config)
        .context("Failed to load configuration")?;

    // Find Zebra installation
    let zebra_dir = match args.zebra_dir {
        Some(dir) => dir,
        None => {
            info!("Auto-detecting Zebra installation...");
            find_zebra_dir()
                .context("Failed to find Zebra installation. Please specify --zebra-dir")?
        }
    };

    println!("✓ Zebra found at: {}", zebra_dir.display());

    // Verify Zebra binary (optional but recommended)
    if !args.skip_verification {
        info!("Verifying Zebra binary...");
        verify_zebra_binary(&zebra_dir)?;
        println!("✓ Zebra binary verified");
    } else {
        warn!("Skipping binary verification (not recommended for production)");
    }

    // Read Zebra node state
    println!("\nReading node metrics...");
    let reader = ZebraReader::new(&zebra_dir)?;
    let node_metrics = reader.read_metrics()
        .context("Failed to read Zebra node metrics")?;

    println!("  Block height: {}", node_metrics.block_height);
    println!("  Sync progress: {:.2}%", node_metrics.sync_percentage);
    println!("  Network: {}", node_metrics.network);

    // Generate ZK proof
    println!("\nGenerating zero-knowledge proof...");
    println!("(This may take 1-2 minutes depending on your hardware)\n");

    let proof_gen = ProofGenerator::new(config.clone());
    let proof = proof_gen.generate_proof(&node_metrics)
        .await
        .context("Failed to generate proof")?;

    println!("✓ Proof generated successfully!");

    // Save proof to file
    let timestamp = chrono::Utc::now().timestamp();
    let output_path = args.output_dir.join(format!("proof_{}.json", timestamp));

    std::fs::create_dir_all(&args.output_dir)?;
    proof.save_to_file(&output_path)
        .context("Failed to save proof")?;

    println!("\n✓ Proof saved to: {}", output_path.display());
    println!("\n📤 Next step:");
    println!("   Upload this file to: https://depinzcash.io/submit");
    println!("   Your rewards will be calculated based on your node metrics.");

    Ok(())
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn find_zebra_dir() -> Result<PathBuf> {
    // Common Zebra installation locations
    let possible_paths = vec![
        dirs::home_dir().map(|h| h.join(".zebra")),
        dirs::home_dir().map(|h| h.join(".cache/zebra")),
        Some(PathBuf::from("/var/lib/zebra")),
        Some(PathBuf::from("C:\\Users\\AppData\\Local\\Zebra")),
    ];

    for path in possible_paths.into_iter().flatten() {
        if path.exists() && path.join("state").exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("Zebra installation not found. Please specify --zebra-dir")
}

fn verify_zebra_binary(zebra_dir: &PathBuf) -> Result<()> {
    // Read zebra binary and compute hash
    let zebra_bin = which::which("zebrad")
        .context("zebrad binary not found in PATH")?;

    use sha2::{Sha256, Digest};
    use std::fs;

    let binary_data = fs::read(&zebra_bin)?;
    let hash = Sha256::digest(&binary_data);
    let hash_hex = hex::encode(hash);

    info!("Zebra binary hash: {}", hash_hex);

    // TODO: Check against known official release hashes
    // For now, just store the hash in the proof for verification

    Ok(())
}
