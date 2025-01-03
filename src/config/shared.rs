use std::sync::Arc;

use crate::config::genesis::{GenesisConfig, ConfigError as GenesisConfigError};
use crate::config::storage::{StorageConfig, ConfigError as StorageConfigError};
use crate::config::tokenomics::{TokenomicsConfig, TokenomicsConfigError};

pub struct SharedConfiguration {
    genesis: Arc<GenesisConfig>,
    storage: Arc<StorageConfig>,
    tokenomics: Arc<TokenomicsConfig>,
}

pub struct SharedConfigError {
    genesis_config_error: Arc<GenesisConfigError>,
    storage_config_error: Arc<StorageConfigError>,
    tokenomics_config_error: Arc<TokenomicsConfigError>,
}