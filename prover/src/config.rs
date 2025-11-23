use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// User configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// User's Solana wallet address for receiving rewards
    pub solana_wallet: String,

    /// User's Zcash address for receiving rewards
    pub zcash_address: String,

    /// Optional custom node identifier
    #[serde(default)]
    pub node_id: Option<String>,

    /// Auto-submit proofs to the website (requires API key)
    #[serde(default)]
    pub auto_submit: bool,

    /// API endpoint for proof submission
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: String,

    /// API key for auto-submission (optional)
    #[serde(default)]
    pub api_key: Option<String>,
}

fn default_api_endpoint() -> String {
    "https://depinzcash.io/api/submit".to_string()
}

impl Config {
    /// Load configuration from a JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read config file: {:?}", path))?;

        let config: Config = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        config.validate()?;

        Ok(config)
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        // Validate Solana wallet address (basic check)
        if self.solana_wallet.is_empty() {
            anyhow::bail!("Solana wallet address cannot be empty");
        }

        // Validate Zcash address (basic check)
        if self.zcash_address.is_empty() {
            anyhow::bail!("Zcash address cannot be empty");
        }

        // Zcash addresses start with 't' (transparent) or 'z' (shielded)
        if !self.zcash_address.starts_with('t') && !self.zcash_address.starts_with('z') {
            anyhow::bail!("Invalid Zcash address format");
        }

        Ok(())
    }

    /// Create a new configuration interactively
    pub fn create_interactive() -> Result<Self> {
        use std::io::{self, Write};

        println!("🔧 DePINZcash Configuration Setup");
        println!("==================================\n");

        print!("Enter your Solana wallet address: ");
        io::stdout().flush()?;
        let mut solana_wallet = String::new();
        io::stdin().read_line(&mut solana_wallet)?;
        let solana_wallet = solana_wallet.trim().to_string();

        print!("Enter your Zcash address (t-addr or z-addr): ");
        io::stdout().flush()?;
        let mut zcash_address = String::new();
        io::stdin().read_line(&mut zcash_address)?;
        let zcash_address = zcash_address.trim().to_string();

        print!("Optional node identifier (press Enter to skip): ");
        io::stdout().flush()?;
        let mut node_id = String::new();
        io::stdin().read_line(&mut node_id)?;
        let node_id = if node_id.trim().is_empty() {
            None
        } else {
            Some(node_id.trim().to_string())
        };

        let config = Config {
            solana_wallet,
            zcash_address,
            node_id,
            auto_submit: false,
            api_endpoint: default_api_endpoint(),
            api_key: None,
        };

        config.validate()?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let valid_config = Config {
            solana_wallet: "8zP3Q...".to_string(),
            zcash_address: "t1abc123...".to_string(),
            node_id: None,
            auto_submit: false,
            api_endpoint: default_api_endpoint(),
            api_key: None,
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = Config {
            solana_wallet: "8zP3Q...".to_string(),
            zcash_address: "invalid".to_string(),
            node_id: None,
            auto_submit: false,
            api_endpoint: default_api_endpoint(),
            api_key: None,
        };

        assert!(invalid_config.validate().is_err());
    }
}
