use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Represents the development runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    /// Deterministic seed for reproducible network simulations
    /// Used to create predictable random sequences in testing
    pub seed: u64,

    /// Block time in seconds for network simulation
    /// Determines the pace of block production during testing
    pub cycle: u32,

    /// Overall operation timeout in milliseconds
    /// Prevents test scenarios from running indefinitely
    pub timeout: u32,
}

/// Represents the production runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    /// Number of threads for parallel processing
    /// Matches minimum node CPU requirements
    pub threads: usize,

    /// Whether to catch and log panics gracefully
    /// Critical for maintaining network stability
    pub catch_panics: bool,

    /// Read operation timeout in milliseconds
    /// Prevents network stalls during data retrieval
    pub read_timeout: u32,

    /// Write operation timeout in milliseconds
    /// Ensures transaction reliability
    pub write_timeout: u32,

    /// TCP_NODELAY configuration
    /// Reduces network latency for consensus messages
    pub tcp_nodelay: bool,

    /// Persistent storage directory
    /// Provides a standard location for blockchain data
    pub storage_directory: PathBuf,

    /// Maximum buffer size for network operations
    /// Balances memory usage and performance
    pub maximum_buffer_size: usize,
}

/// Enumeration of possible runtime environments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ValueEnum)]  // Added ValueEnum
pub enum RuntimeEnvironment {
    Development,
    Production,
}

/// Comprehensive runtime configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Current runtime environment
    pub environment: RuntimeEnvironment,

    /// Development-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub development: Option<DevelopmentConfig>,

    /// Production-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production: Option<ProductionConfig>,
}

/// Error types for runtime configuration
#[derive(Error, Debug)]
pub enum RuntimeConfigError {
    #[error("Configuration file not found")]
    FileNotFound(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Missing environment-specific configuration")]
    MissingEnvironmentConfig,
}

impl RuntimeConfig {
    /// Load default runtime configuration
    /// Attempts to read from standard locations with fallback mechanisms
    pub fn load_default() -> Result<Self, RuntimeConfigError> {
        // Potential configuration file paths
        let config_paths = vec![
            PathBuf::from("/etc/romer/runtime.toml"),
            PathBuf::from("./config/runtime.toml"),
            PathBuf::from("./runtime.toml"),
        ];

        // Try each potential path
        for path in config_paths {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        // If no file found, provide sensible defaults
        Self::generate_default_config()
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, RuntimeConfigError> {
        let config_content = fs::read_to_string(path).map_err(RuntimeConfigError::FileNotFound)?;

        toml::from_str(&config_content)
            .map_err(|e| RuntimeConfigError::InvalidConfig(e.to_string()))
    }

    /// Generate a default configuration
    fn generate_default_config() -> Result<Self, RuntimeConfigError> {
        // Default to development environment if not explicitly configured
        Ok(Self {
            environment: RuntimeEnvironment::Development,
            development: Some(DevelopmentConfig {
                seed: 42,
                cycle: 30,
                timeout: 60000,
            }),
            production: Some(ProductionConfig {
                threads: 8,
                catch_panics: true,
                read_timeout: 5000,
                write_timeout: 5000,
                tcp_nodelay: true,
                storage_directory: env::temp_dir().join("romer_storage"),
                maximum_buffer_size: 67_108_864, // 64MB
            }),
        })
    }

    /// Validate the configuration based on environment
    pub fn validate(&self) -> Result<(), RuntimeConfigError> {
        match self.environment {
            RuntimeEnvironment::Development => {
                if self.development.is_none() {
                    return Err(RuntimeConfigError::MissingEnvironmentConfig);
                }
            }
            RuntimeEnvironment::Production => {
                if self.production.is_none() {
                    return Err(RuntimeConfigError::MissingEnvironmentConfig);
                }
            }
        }
        Ok(())
    }

    /// Get the active configuration based on environment
    pub fn get_active_config(&self) -> Result<Box<dyn std::fmt::Debug>, RuntimeConfigError> {
        match self.environment {
            RuntimeEnvironment::Development => self
                .development
                .clone()
                .map(|config| Box::new(config) as Box<dyn std::fmt::Debug>)
                .ok_or(RuntimeConfigError::MissingEnvironmentConfig),
            RuntimeEnvironment::Production => self
                .production
                .clone()
                .map(|config| Box::new(config) as Box<dyn std::fmt::Debug>)
                .ok_or(RuntimeConfigError::MissingEnvironmentConfig),
        }
    }
}
