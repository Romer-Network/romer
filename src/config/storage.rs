use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

/// Error type for storage configuration operations
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

/// The main storage configuration structure
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StorageConfig {
    pub metadata: MetadataConfig,
    pub journal: JournalConfig,
    pub paths: PathConfig,
    pub backup: BackupConfig,
}

/// Configuration for metadata storage partitions
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetadataConfig {
    pub validator_partition: String,
    pub region_partition: String,
    pub network_partition: String,
    pub sync_interval_ms: u64,
    pub max_batch_size: usize,
}

/// Configuration for journal-based block storage
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JournalConfig {
    pub blocks_per_section: u64,
    pub partitions: JournalPartitions,
    pub retention: RetentionPolicy,
    pub performance: PerformanceConfig,
}

/// Names for different journal partitions
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JournalPartitions {
    pub genesis: String,
    pub blocks: String,
    pub transactions: String,
    pub receipts: String,
}

/// Configuration for data retention
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RetentionPolicy {
    pub minimum_sections: u64,
    pub max_age_days: u32,
}

/// Performance tuning parameters
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PerformanceConfig {
    pub replay_concurrency: usize,
    pub pending_writes: usize,
    pub compression_level: i32,
}

/// Storage path configuration
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PathConfig {
    pub data_dir: PathBuf,
    pub metadata_dir: PathBuf,
    pub journal_dir: PathBuf,
    pub archive_dir: PathBuf,
}

/// Backup configuration
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BackupConfig {
    pub enabled: bool,
    pub interval_hours: u32,
    pub retention_days: u32,
}

/// Default values for configuration parameters
pub mod defaults {
    pub const BLOCKS_PER_SECTION: u64 = 1000;
    pub const MINIMUM_SECTIONS: u64 = 100;
    pub const MAX_AGE_DAYS: u32 = 30;
    pub const SYNC_INTERVAL_MS: u64 = 5000;
    pub const MAX_BATCH_SIZE: usize = 1000;
    pub const REPLAY_CONCURRENCY: usize = 4;
    pub const PENDING_WRITES: usize = 10;
    pub const COMPRESSION_LEVEL: i32 = 3;
}

impl StorageConfig {
    /// Loads the configuration from the default location
    pub fn load_default() -> Result<Self, ConfigError> {
        let config_path = Self::default_config_path()?;
        Self::load(&config_path)
    }

    /// Loads the configuration from a specific path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: StorageConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Determines the default configuration path
    fn default_config_path() -> Result<PathBuf, ConfigError> {
        // First check if path is specified in environment
        if let Ok(path) = env::var("ROMER_STORAGE_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        // Then check in the config directory
        let config_dir = PathBuf::from("config");
        
        // Check for environment-specific config first
        let env = env::var("ROMER_ENV").unwrap_or_else(|_| "development".to_string());
        let env_specific_path = config_dir.join(format!("storage.{}.toml", env));
        if env_specific_path.exists() {
            return Ok(env_specific_path);
        }

        // Fall back to default config
        let default_path = config_dir.join("storage.toml");
        if default_path.exists() {
            return Ok(default_path);
        }

        Err(ConfigError::ValidationError(
            "Could not find storage configuration file".to_string()
        ))
    }

    /// Creates a development configuration with default values
    pub fn development() -> Self {
        Self {
            metadata: MetadataConfig {
                validator_partition: "validators".to_string(),
                region_partition: "regions".to_string(),
                network_partition: "network_state".to_string(),
                sync_interval_ms: defaults::SYNC_INTERVAL_MS,
                max_batch_size: defaults::MAX_BATCH_SIZE,
            },
            journal: JournalConfig {
                blocks_per_section: defaults::BLOCKS_PER_SECTION,
                partitions: JournalPartitions {
                    genesis: "genesis_data".to_string(),
                    blocks: "block_data".to_string(),
                    transactions: "tx_data".to_string(),
                    receipts: "receipt_data".to_string(),
                },
                retention: RetentionPolicy {
                    minimum_sections: defaults::MINIMUM_SECTIONS,
                    max_age_days: defaults::MAX_AGE_DAYS,
                },
                performance: PerformanceConfig {
                    replay_concurrency: defaults::REPLAY_CONCURRENCY,
                    pending_writes: defaults::PENDING_WRITES,
                    compression_level: defaults::COMPRESSION_LEVEL,
                },
            },
            paths: PathConfig {
                data_dir: PathBuf::from("data"),
                metadata_dir: PathBuf::from("data/metadata"),
                journal_dir: PathBuf::from("data/journal"),
                archive_dir: PathBuf::from("data/archive"),
            },
            backup: BackupConfig {
                enabled: true,
                interval_hours: 24,
                retention_days: 7,
            },
        }
    }

    /// Validates the configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate metadata configuration
        if self.metadata.sync_interval_ms < 1000 {
            return Err(ConfigError::ValidationError(
                "Metadata sync interval must be at least 1000ms".to_string()
            ));
        }

        // Validate journal configuration
        if self.journal.blocks_per_section < 100 || self.journal.blocks_per_section > 10000 {
            return Err(ConfigError::ValidationError(
                "Blocks per section must be between 100 and 10000".to_string()
            ));
        }

        if self.journal.retention.minimum_sections < 10 {
            return Err(ConfigError::ValidationError(
                "Minimum sections must be at least 10".to_string()
            ));
        }

        if self.journal.performance.replay_concurrency == 0 {
            return Err(ConfigError::ValidationError(
                "Replay concurrency must be greater than 0".to_string()
            ));
        }

        if self.journal.performance.compression_level < -1 || self.journal.performance.compression_level > 9 {
            return Err(ConfigError::ValidationError(
                "Compression level must be between -1 and 9".to_string()
            ));
        }

        // Validate backup configuration
        if self.backup.enabled && self.backup.interval_hours == 0 {
            return Err(ConfigError::ValidationError(
                "Backup interval must be greater than 0 hours when enabled".to_string()
            ));
        }

        Ok(())
    }

    /// Creates required directories based on the path configuration
    pub fn initialize_directories(&self) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.paths.data_dir)?;
        fs::create_dir_all(&self.paths.metadata_dir)?;
        fs::create_dir_all(&self.paths.journal_dir)?;
        fs::create_dir_all(&self.paths.archive_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let config = StorageConfig::development();
        assert_eq!(config.journal.blocks_per_section, defaults::BLOCKS_PER_SECTION);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation() {
        let mut config = StorageConfig::development();
        
        // Test invalid blocks per section
        config.journal.blocks_per_section = 50;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValidationError(_))
        ));

        // Test invalid sync interval
        let mut config = StorageConfig::development();
        config.metadata.sync_interval_ms = 500;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValidationError(_))
        ));

        // Test invalid compression level
        let mut config = StorageConfig::development();
        config.journal.performance.compression_level = 10;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValidationError(_))
        ));
    }
}