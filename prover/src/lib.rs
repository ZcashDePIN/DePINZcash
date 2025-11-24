// Library exports for testing and external use

pub mod config;
pub mod proof_gen;
pub mod zebra_reader;

// Re-export commonly used types
pub use config::Config;
pub use proof_gen::{Proof, ProofGenerator, ProofMetrics, NodeInfo, RewardCalculation};
pub use zebra_reader::{NodeMetrics, ZebraReader};
