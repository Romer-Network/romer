use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

/// Error type for tokenomics configuration operations
#[derive(Debug)]
pub enum TokenomicsConfigError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    ValidationError(String),
}

impl std::fmt::Display for TokenomicsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenomicsConfigError::IoError(e) => write!(f, "IO error: {}", e),
            TokenomicsConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            TokenomicsConfigError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for TokenomicsConfigError {}

impl From<std::io::Error> for TokenomicsConfigError {
    fn from(error: std::io::Error) -> Self {
        TokenomicsConfigError::IoError(error)
    }
}

impl From<toml::de::Error> for TokenomicsConfigError {
    fn from(error: toml::de::Error) -> Self {
        TokenomicsConfigError::ParseError(error)
    }
}

/// Token configuration including name, symbol, and decimal places
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub smallest_unit_name: String,
}

/// Supply configuration defining initial token supply
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupplyConfig {
    pub initial_supply: u64,
}

/// Address configuration for system-critical addresses
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressConfig {
    pub treasury: String,
    pub burn: String,
}

/// Distribution configuration for initial token allocation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DistributionConfig {
    pub treasury_allocation: u64,
}

/// Block rewards configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockRewardsConfig {
    pub base_reward: u64,
}

/// Network utilization thresholds for monetary policy
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UtilizationThresholds {
    pub low: u32,
    pub high: u32,
}

/// Reward adjustments based on network utilization
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardAdjustments {
    pub burn: i64,
    pub mint: i64,
}

/// Network policy configuration for dynamic monetary adjustments
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkPolicyConfig {
    pub target_transactions_per_block: u32,
    pub adjustment_period_blocks: u32,
    pub utilization_thresholds: UtilizationThresholds,
    pub reward_adjustments: RewardAdjustments,
}

/// Network metrics configuration for monitoring and adjustments
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkMetricsConfig {
    pub metrics_window_blocks: u32,
    pub min_blocks_for_adjustment: u32,
    pub update_frequency_blocks: u32,
}

/// Main tokenomics configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenomicsConfig {
    pub token: TokenConfig,
    pub supply: SupplyConfig,
    pub addresses: AddressConfig,
    pub distribution: DistributionConfig,
    pub block_rewards: BlockRewardsConfig,
    pub network_policy: NetworkPolicyConfig,
    pub network_metrics: NetworkMetricsConfig,
}

/// Default values for configuration parameters
pub mod defaults {
    pub const DECIMALS: u8 = 2;
    pub const TOKEN_NAME: &str = "RØMER";
    pub const TOKEN_SYMBOL: &str = "ROMER";
    pub const SMALLEST_UNIT_NAME: &str = "Ole";
    pub const INITIAL_SUPPLY: u64 = 30000000;  // 300,000 RØMER in Ole units
    pub const BASE_BLOCK_REWARD: u64 = 100;    // 1 RØMER per block in Ole units
    pub const TARGET_TXS_PER_BLOCK: u32 = 50;
    pub const ADJUSTMENT_PERIOD_BLOCKS: u32 = 10080; // One week (7 * 24 * 60)
    pub const METRICS_WINDOW_BLOCKS: u32 = 10080;    // One week of blocks
    pub const MIN_BLOCKS_FOR_ADJUSTMENT: u32 = 5040; // Half week minimum
    pub const TREASURY_ADDRESS: &str = "3eec2d691ee2952ff9924a0db1db24c356d38a8e16b0e4b2b6f1a6a15588e112";
    pub const BURN_ADDRESS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
}

impl TokenomicsConfig {
    /// Loads the configuration from the default location
    pub fn load_default() -> Result<Self, TokenomicsConfigError> {
        let config_path = Self::default_config_path()?;
        Self::load(&config_path)
    }

    /// Loads the configuration from a specific path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, TokenomicsConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: TokenomicsConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Determines the default configuration path
    fn default_config_path() -> Result<PathBuf, TokenomicsConfigError> {
        if let Ok(path) = env::var("ROMER_TOKENOMICS_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        let config_dir = PathBuf::from("config");
        
        let env = env::var("ROMER_ENV").unwrap_or_else(|_| "development".to_string());
        let env_specific_path = config_dir.join(format!("tokenomics.{}.toml", env));
        if env_specific_path.exists() {
            return Ok(env_specific_path);
        }

        let default_path = config_dir.join("tokenomics.toml");
        if default_path.exists() {
            return Ok(default_path);
        }

        Err(TokenomicsConfigError::ValidationError(
            "Could not find tokenomics configuration file".to_string()
        ))
    }

    /// Creates a development configuration with default values
    pub fn development() -> Self {
        Self {
            token: TokenConfig {
                name: defaults::TOKEN_NAME.to_string(),
                symbol: defaults::TOKEN_SYMBOL.to_string(),
                decimals: defaults::DECIMALS,
                smallest_unit_name: defaults::SMALLEST_UNIT_NAME.to_string(),
            },
            supply: SupplyConfig {
                initial_supply: defaults::INITIAL_SUPPLY,
            },
            addresses: AddressConfig {
                treasury: defaults::TREASURY_ADDRESS.to_string(),
                burn: defaults::BURN_ADDRESS.to_string(),
            },
            distribution: DistributionConfig {
                treasury_allocation: defaults::INITIAL_SUPPLY,
            },
            block_rewards: BlockRewardsConfig {
                base_reward: defaults::BASE_BLOCK_REWARD,
            },
            network_policy: NetworkPolicyConfig {
                target_transactions_per_block: defaults::TARGET_TXS_PER_BLOCK,
                adjustment_period_blocks: defaults::ADJUSTMENT_PERIOD_BLOCKS,
                utilization_thresholds: UtilizationThresholds {
                    low: 25,
                    high: 100,
                },
                reward_adjustments: RewardAdjustments {
                    burn: -100,
                    mint: 100,
                },
            },
            network_metrics: NetworkMetricsConfig {
                metrics_window_blocks: defaults::METRICS_WINDOW_BLOCKS,
                min_blocks_for_adjustment: defaults::MIN_BLOCKS_FOR_ADJUSTMENT,
                update_frequency_blocks: defaults::ADJUSTMENT_PERIOD_BLOCKS,
            },
        }
    }

    /// Validates the configuration values
    fn validate(&self) -> Result<(), TokenomicsConfigError> {
        if self.token.decimals != defaults::DECIMALS {
            return Err(TokenomicsConfigError::ValidationError(
                format!("Token decimals must be {}", defaults::DECIMALS)
            ));
        }

        if self.supply.initial_supply != self.distribution.treasury_allocation {
            return Err(TokenomicsConfigError::ValidationError(
                "Initial supply must match treasury allocation".to_string()
            ));
        }

        if self.network_policy.utilization_thresholds.high <= self.network_policy.utilization_thresholds.low {
            return Err(TokenomicsConfigError::ValidationError(
                "High utilization threshold must be greater than low threshold".to_string()
            ));
        }

        if self.block_rewards.base_reward == 0 {
            return Err(TokenomicsConfigError::ValidationError(
                "Base block reward cannot be zero".to_string()
            ));
        }

        if self.network_metrics.min_blocks_for_adjustment >= self.network_metrics.metrics_window_blocks {
            return Err(TokenomicsConfigError::ValidationError(
                "Minimum blocks for adjustment must be less than metrics window".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let config = TokenomicsConfig::development();
        assert_eq!(config.token.decimals, defaults::DECIMALS);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation() {
        let mut config = TokenomicsConfig::development();
        
        // Test invalid decimals
        config.token.decimals = 8;
        assert!(matches!(
            config.validate(),
            Err(TokenomicsConfigError::ValidationError(_))
        ));

        // Test mismatched supply and allocation
        let mut config = TokenomicsConfig::development();
        config.distribution.treasury_allocation = config.supply.initial_supply + 1;
        assert!(matches!(
            config.validate(),
            Err(TokenomicsConfigError::ValidationError(_))
        ));

        // Test invalid thresholds
        let mut config = TokenomicsConfig::development();
        config.network_policy.utilization_thresholds.high = 
            config.network_policy.utilization_thresholds.low;
        assert!(matches!(
            config.validate(),
            Err(TokenomicsConfigError::ValidationError(_))
        ));
    }
}