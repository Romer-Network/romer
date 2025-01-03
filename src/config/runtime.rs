use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::time::Duration;
use thiserror::Error;

/// Comprehensive runtime configuration for Rømer Chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeConfig {
    /// Execution environment configuration
    pub environment: ExecutionEnvironment,

    /// Task execution and scheduling parameters
    pub executor: ExecutorConfig,

    /// Network-related runtime configurations
    pub network: NetworkConfig,

    /// Storage and persistence runtime settings
    pub storage: StorageConfig,

    /// Performance and resource management
    pub performance: PerformanceConfig,

    /// Fault tolerance and recovery mechanisms
    pub fault_tolerance: FaultToleranceConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Metrics and observability settings
    pub metrics: MetricsConfig,

    /// Deterministic runtime modes
    pub deterministic: DeterministicConfig,
}

/// Execution environment specification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ExecutionEnvironment {
    Production,
    Testing,
    Development,
}

/// Task executor configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutorConfig {
    /// Default timeout for task execution
    pub default_timeout_ms: u64,

    /// Maximum number of retries for a failed task
    pub max_task_retries: u8,

    /// Delay between task retries
    pub task_retry_delay_ms: u64,

    /// Task queuing and scheduling strategy
    pub scheduling_strategy: SchedulingStrategy,
}

/// Task scheduling strategies
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SchedulingStrategy {
    Static,
    Dynamic,
    Adaptive,
}

/// Network runtime configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Connection establishment timeout
    pub connection_timeout_ms: u64,

    /// Maximum concurrent network connections
    pub max_concurrent_connections: u32,

    /// Keepalive interval for persistent connections
    pub keepalive_interval_ms: u64,

    /// Network message size limits
    pub max_message_size_bytes: usize,
}

/// Storage runtime configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Default storage partition
    pub default_partition: String,

    /// Maximum number of open blob handles
    pub max_open_blobs: u32,

    /// Interval for synchronizing storage
    pub blob_sync_interval_ms: u64,

    /// Default compression level for storage
    pub default_compression_level: i32,
}

/// Performance and resource management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent task spawning
    pub max_spawn_concurrency: u32,

    /// Size of the task spawn queue
    pub spawn_queue_size: u32,

    /// Resource utilization thresholds
    pub resource_thresholds: ResourceThresholds,
}

/// Resource utilization thresholds
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceThresholds {
    /// CPU utilization threshold for scaling
    pub cpu_utilization_threshold: f32,

    /// Memory utilization threshold for scaling
    pub memory_utilization_threshold: f32,

    /// Network bandwidth utilization threshold
    pub network_utilization_threshold: f32,
}

/// Fault tolerance and recovery configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FaultToleranceConfig {
    /// Maximum number of task failures before intervention
    pub max_task_failures: u8,

    /// Enable automatic recovery mechanisms
    pub auto_recovery_enabled: bool,

    /// Recovery strategy when failures occur
    pub recovery_strategy: RecoveryStrategy,
}

/// Recovery strategies for task failures
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RecoveryStrategy {
    Restart,
    Fallback,
    Abort,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Logging verbosity level
    pub log_level: LogLevel,

    /// Log output format
    pub log_format: LogFormat,

    /// Maximum log file size in megabytes
    pub max_log_file_size_mb: u32,

    /// Maximum number of log files to retain
    pub max_log_files: u8,
}

/// Logging verbosity levels
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Log output formats
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LogFormat {
    Plain,
    Json,
    Compact,
}

/// Metrics and observability configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,

    /// Port for exposing metrics
    pub metrics_port: u16,

    /// Endpoint path for metrics
    pub metrics_path: String,

    /// Namespace for metrics
    pub prometheus_namespace: String,

    /// Metrics collection interval
    pub collection_interval_ms: u64,
}

/// Deterministic runtime configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeterministicConfig {
    /// Seed for reproducible random number generation
    pub seed: u64,

    /// Maximum number of deterministic tasks
    pub max_deterministic_tasks: u32,

    /// Randomness generation strategy
    pub randomness_strategy: RandomnessStrategy,
}

/// Strategies for generating randomness
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RandomnessStrategy {
    TrueRandom,
    Seeded,
    PseudoDeterministic,
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

impl RuntimeConfig {
    /// Load configuration from default location
    pub fn load_default() -> Result<Self, ConfigError> {
        let config_path = Self::default_config_path()?;
        Self::load(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: RuntimeConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Determine the default configuration path
    fn default_config_path() -> Result<PathBuf, ConfigError> {
        // Check environment variable first
        if let Ok(path) = env::var("ROMER_RUNTIME_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        let config_dir = PathBuf::from("config");
        
        // Check environment-specific config
        let env = env::var("ROMER_ENV").unwrap_or_else(|_| "development".to_string());
        let env_specific_path = config_dir.join(format!("runtime.{}.toml", env));
        if env_specific_path.exists() {
            return Ok(env_specific_path);
        }

        // Fallback to default config
        let default_path = config_dir.join("runtime.toml");
        if default_path.exists() {
            return Ok(default_path);
        }

        Err(ConfigError::ValidationError(
            "Could not find runtime configuration file".to_string()
        ))
    }

    /// Validate configuration parameters
    fn validate(&self) -> Result<(), ConfigError> {
        // Executor configuration validation
        if self.executor.default_timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Default timeout must be greater than 0".to_string()
            ));
        }

        // Network configuration validation
        if self.network.max_concurrent_connections == 0 {
            return Err(ConfigError::ValidationError(
                "Maximum concurrent connections must be greater than 0".to_string()
            ));
        }

        // Resource threshold validation
        if self.performance.resource_thresholds.cpu_utilization_threshold > 1.0 ||
           self.performance.resource_thresholds.memory_utilization_threshold > 1.0 ||
           self.performance.resource_thresholds.network_utilization_threshold > 1.0 {
            return Err(ConfigError::ValidationError(
                "Resource utilization thresholds cannot exceed 1.0".to_string()
            ));
        }

        // Logging configuration validation
        if self.logging.max_log_files == 0 {
            return Err(ConfigError::ValidationError(
                "At least one log file must be retained".to_string()
            ));
        }

        Ok(())
    }

    /// Create a development configuration
    pub fn development() -> Self {
        Self {
            environment: ExecutionEnvironment::Development,
            executor: ExecutorConfig {
                default_timeout_ms: 30_000,
                max_task_retries: 3,
                task_retry_delay_ms: 1_000,
                scheduling_strategy: SchedulingStrategy::Dynamic,
            },
            network: NetworkConfig {
                connection_timeout_ms: 5_000,
                max_concurrent_connections: 50,
                keepalive_interval_ms: 30_000,
                max_message_size_bytes: 1_024 * 1024, // 1 MB
            },
            storage: StorageConfig {
                default_partition: "dev".to_string(),
                max_open_blobs: 10,
                blob_sync_interval_ms: 5_000,
                default_compression_level: 3,
            },
            performance: PerformanceConfig {
                max_spawn_concurrency: 4,
                spawn_queue_size: 128,
                resource_thresholds: ResourceThresholds {
                    cpu_utilization_threshold: 0.75,
                    memory_utilization_threshold: 0.8,
                    network_utilization_threshold: 0.7,
                },
            },
            fault_tolerance: FaultToleranceConfig {
                max_task_failures: 5,
                auto_recovery_enabled: true,
                recovery_strategy: RecoveryStrategy::Restart,
            },
            logging: LoggingConfig {
                log_level: LogLevel::Debug,
                log_format: LogFormat::Json,
                max_log_file_size_mb: 50,
                max_log_files: 5,
            },
            metrics: MetricsConfig {
                metrics_enabled: true,
                metrics_port: 9000,
                metrics_path: "/metrics".to_string(),
                prometheus_namespace: "romer_runtime".to_string(),
                collection_interval_ms: 15_000,
            },
            deterministic: DeterministicConfig {
                seed: 0, // 0 for true randomness
                max_deterministic_tasks: 100,
                randomness_strategy: RandomnessStrategy::TrueRandom,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let config = RuntimeConfig::development();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        // Test various validation scenarios
        let mut config = RuntimeConfig::development();
        
        // Test timeout validation
        config.executor.default_timeout_ms = 0;
        assert!(config.validate().is_err());

        // Test resource threshold validation
        config = RuntimeConfig::development();
        config.performance.resource_thresholds.cpu_utilization_threshold = 1.5;
        assert!(config.validate().is_err());
    }
}