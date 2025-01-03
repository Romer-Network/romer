use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Error type for genesis configuration operations
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConfigError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        ConfigError::ParseError(error)
    }
}

impl From<String> for ConfigError {
    fn from(error: String) -> Self {
        ConfigError::ValidationError(error)
    }
}

/// The main configuration structure for the genesis block and network parameters
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenesisConfig {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub networking: NetworkingConfig,
    pub technical: TechnicalConfig,
}

/// Basic network identification parameters
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NetworkConfig {
    pub chain_id: String,
    pub version: String,
    pub genesis_time: u64,
}

/// Parameters that control how consensus operates
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConsensusConfig {
    pub block_time_ms: u64,
    pub epoch_length: u64,
    pub min_validators: u32,
    pub max_validators: u32,
}

/// Configuration for the peer-to-peer networking layer
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NetworkingConfig {
    pub max_peers: u32,
    pub max_message_size: usize,
    pub max_message_backlog: usize,
    pub compression_level: u8,
    pub connection_timeout_ms: u32,
    pub peer_discovery_interval: u32,
}

/// Technical limitations and parameters for the blockchain
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TechnicalConfig {
    pub max_block_size: u32,
    pub max_tx_size: u32,
}


impl GenesisConfig {
    /// Loads the configuration from the default location
    pub fn load_default() -> Result<Self, ConfigError> {
        let config_path = Self::default_config_path()?;
        Self::load(&config_path)
    }

    /// Loads the configuration from a specific path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: GenesisConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Determines the default configuration path
    fn default_config_path() -> Result<PathBuf, ConfigError> {
        // First check if path is specified in environment
        if let Ok(path) = env::var("ROMER_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        // Then check in the config directory relative to the project root
        let config_dir = PathBuf::from("config");

        // Check for environment-specific config first
        let env = env::var("ROMER_ENV").unwrap_or_else(|_| "development".to_string());
        let env_specific_path = config_dir.join(format!("genesis.{}.toml", env));
        if env_specific_path.exists() {
            return Ok(env_specific_path);
        }

        // Fall back to default config
        let default_path = config_dir.join("genesis.toml");
        if default_path.exists() {
            return Ok(default_path);
        }

        Err(ConfigError::ValidationError(
            "Could not find configuration file".to_string(),
        ))
    }

    /// Validates the configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate network configuration
        if self.network.chain_id.is_empty() {
            return Err(ConfigError::ValidationError(
                "Chain ID cannot be empty".to_string(),
            ));
        }

        if self.network.version.is_empty() {
            return Err(ConfigError::ValidationError(
                "Version cannot be empty".to_string(),
            ));
        }

        // Validate consensus configuration
        if self.consensus.block_time_ms < 100 || self.consensus.block_time_ms > 10_000 {
            return Err(ConfigError::ValidationError(
                "Block time must be between 100ms and 10 seconds".to_string(),
            ));
        }

        if self.consensus.epoch_length < 10 {
            return Err(ConfigError::ValidationError(
                "Epoch length must be at least 10 blocks".to_string(),
            ));
        }

        if self.consensus.max_validators < self.consensus.min_validators {
            return Err(ConfigError::ValidationError(
                "Maximum validators must be greater than minimum validators".to_string(),
            ));
        }

        // Validate networking configuration
        if self.networking.max_message_size > 10 * 1024 * 1024 {
            return Err(ConfigError::ValidationError(
                "Maximum message size cannot exceed 10MB".to_string(),
            ));
        }

        if self.networking.connection_timeout_ms < 1000 {
            return Err(ConfigError::ValidationError(
                "Connection timeout must be at least 1000ms".to_string(),
            ));
        }

        // Validate technical configuration
        if self.technical.max_block_size <= self.technical.max_tx_size {
            return Err(ConfigError::ValidationError(
                "Maximum block size must be greater than maximum transaction size".to_string(),
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
        let config = GenesisConfig::development();
        assert_eq!(config.network.chain_id, "romer-dev");
        assert_eq!(config.network.version, "0.1.0");
        assert!(config.network.genesis_time > 0);
    }

    #[test]
    fn test_validation() {
        let mut config = GenesisConfig::development();
        assert!(config.validate().is_ok());

        // Test invalid block time
        config.consensus.block_time_ms = 50;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValidationError(_))
        ));

        // Reset and test invalid message size
        config = GenesisConfig::development();
        config.networking.max_message_size = 20 * 1024 * 1024;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValidationError(_))
        ));

        // Reset and test invalid block/tx size relationship
        config = GenesisConfig::development();
        config.technical.max_block_size = 1000;
        config.technical.max_tx_size = 2000;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValidationError(_))
        ));
    }

    #[test]
    fn test_serialization() {
        let config = GenesisConfig::development();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: GenesisConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.network.chain_id, deserialized.network.chain_id);
        assert_eq!(
            config.consensus.block_time_ms,
            deserialized.consensus.block_time_ms
        );
        assert_eq!(
            config.networking.max_peers,
            deserialized.networking.max_peers
        );
        assert_eq!(
            config.technical.max_block_size,
            deserialized.technical.max_block_size
        );
    }
}
