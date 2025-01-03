use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use thiserror::Error;

/// Comprehensive application configuration for Rømer Chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplicationConfig {
    /// Network configuration parameters
    pub network: NetworkConfig,

    /// Consensus mechanism configuration
    pub consensus: ConsensusConfig,

    /// Tokenomics and economic parameters
    pub tokenomics: TokenomicsConfig,

    /// Storage and persistence configuration
    pub storage: StorageConfig,

    /// Validator requirements and parameters
    pub validators: ValidatorConfig,

    /// Infrastructure and performance settings
    pub infrastructure: InfrastructureConfig,
}

/// Network-level configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Unique identifier for the chain
    pub chain_id: String,

    /// Current protocol version
    pub version: String,

    /// Genesis timestamp
    pub genesis_timestamp: u64,

    /// Target block time in seconds
    pub block_time_seconds: u32,

    /// Number of blocks per epoch
    pub epoch_length_blocks: u32,

    /// Maximum message size
    pub max_message_size_bytes: usize,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u32,
}

/// Consensus mechanism configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsensusConfig {
    /// Minimum number of validators required
    pub min_validators: u32,

    /// Maximum number of validators allowed
    pub max_validators: u32,

    /// Block size limits
    pub max_block_size_bytes: u32,

    /// Maximum transactions per block
    pub max_transactions_per_block: u32,

    /// Total computational resources per block
    pub max_block_gas_limit: u64,
}

/// Tokenomics and economic parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenomicsConfig {
    /// Token metadata
    pub token: TokenDetails,

    /// Initial supply configuration
    pub supply: SupplyParameters,

    /// Reward and fee structures
    pub rewards: RewardConfig,

    /// Monetary policy parameters
    pub monetary_policy: MonetaryPolicyConfig,
}

/// Detailed token information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenDetails {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub smallest_unit: String,
}

/// Supply and distribution parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupplyParameters {
    pub initial_supply: u64,
    pub treasury_allocation: u64,
    pub burn_address: String,
}

/// Reward and fee configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardConfig {
    pub base_block_reward: u64,
    pub transaction_fee_minimum: u64,
    pub storage_deposit_per_byte: u64,
}

/// Monetary policy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonetaryPolicyConfig {
    pub reward_adjustment_period_blocks: u32,
    pub min_blocks_for_adjustment: u32,
    pub utilization_low_threshold: u32,
    pub utilization_high_threshold: u32,
    pub max_reward_adjustment_percent: u32,
}

/// Storage and persistence configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Paths for different storage components
    pub paths: StoragePaths,

    /// Journal and block storage configuration
    pub journal: JournalConfig,

    /// Metadata storage parameters
    pub metadata: MetadataConfig,

    /// Backup and retention policies
    pub backup: BackupConfig,
}

/// Storage paths configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoragePaths {
    pub data_dir: PathBuf,
    pub metadata_dir: PathBuf,
    pub journal_dir: PathBuf,
    pub archive_dir: PathBuf,
}

/// Journal storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JournalConfig {
    pub blocks_per_section: u64,
    pub replay_concurrency: usize,
    pub pending_writes: usize,
    pub compression_level: i32,
}

/// Metadata storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetadataConfig {
    pub sync_interval_ms: u64,
    pub max_batch_size: usize,
}

/// Backup configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub interval_hours: u32,
    pub retention_days: u32,
}

/// Validator requirements and parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorConfig {
    /// Hardware requirements
    pub hardware: HardwareRequirements,

    /// Network and geographic requirements
    pub network: NetworkRequirements,

    /// Performance expectations
    pub performance: PerformanceRequirements,
}

/// Validator hardware requirements
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HardwareRequirements {
    pub min_cpu_cores: u32,
    pub min_ram_gb: u32,
    pub min_storage_tb: u32,
    pub min_bandwidth_mbps: u32,
}

/// Validator network requirements
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkRequirements {
    pub min_regions: u32,
    pub max_validators_per_region: u32,
    pub min_region_distance_km: u32,
    pub max_latency_ms: u32,
    pub required_path_diversity: u32,
}

/// Validator performance requirements
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceRequirements {
    pub min_uptime_percent: f32,
    pub performance_evaluation_blocks: u32,
    pub max_missed_blocks: u32,
    pub max_response_time_ms: u32,
}

/// Infrastructure and performance settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InfrastructureConfig {
    /// Hardware verification interval
    pub hardware_verification_blocks: u32,

    /// Network path verification interval
    pub network_path_verification_blocks: u32,

    /// Geographic reverification interval
    pub geographic_reverification_blocks: u32,

    /// Location proof threshold
    pub location_proof_threshold_metres: u32,

    /// Minimum infrastructure proof complexity
    pub min_infrastructure_proof_bits: u32,
}

/// Configuration loading and validation errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(#[from] toml::de::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl ApplicationConfig {
    /// Load configuration from default location
    pub fn load_default() -> Result<Self, ConfigError> {
        let config_path = Self::default_config_path()?;
        Self::load(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: ApplicationConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Determine the default configuration path
    fn default_config_path() -> Result<PathBuf, ConfigError> {
        // Check environment variable first
        if let Ok(path) = env::var("ROMER_APP_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        let config_dir = PathBuf::from("config");
        
        // Check environment-specific config
        let env = env::var("ROMER_ENV").unwrap_or_else(|_| "development".to_string());
        let env_specific_path = config_dir.join(format!("application.{}.toml", env));
        if env_specific_path.exists() {
            return Ok(env_specific_path);
        }

        // Fallback to default config
        let default_path = config_dir.join("application.toml");
        if default_path.exists() {
            return Ok(default_path);
        }

        Err(ConfigError::ValidationError(
            "Could not find application configuration file".to_string()
        ))
    }

    /// Validate configuration parameters
    fn validate(&self) -> Result<(), ConfigError> {
        // Network validation
        if self.network.block_time_seconds == 0 {
            return Err(ConfigError::ValidationError(
                "Block time must be greater than 0".to_string()
            ));
        }

        // Consensus validation
        if self.consensus.min_validators > self.consensus.max_validators {
            return Err(ConfigError::ValidationError(
                "Minimum validators cannot exceed maximum validators".to_string()
            ));
        }

        // Tokenomics validation
        if self.tokenomics.supply.initial_supply == 0 {
            return Err(ConfigError::ValidationError(
                "Initial supply must be greater than 0".to_string()
            ));
        }

        // Validator hardware validation
        if self.validators.hardware.min_ram_gb < 32 {
            return Err(ConfigError::ValidationError(
                "Minimum RAM must be at least 32GB".to_string()
            ));
        }

        Ok(())
    }

    /// Create a development configuration
    pub fn development() -> Self {
        // Implementation with sensible development defaults
        // This would mirror the approach in your existing configs
        todo!("Implement development configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let config = ApplicationConfig::development();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        // Add tests for various validation scenarios
        // Similar to your existing config test modules
    }
}