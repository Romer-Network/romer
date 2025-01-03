use std::sync::Arc;

use crate::config::genesis::{ConfigError as GenesisConfigError, GenesisConfig};
use crate::config::storage::{ConfigError as StorageConfigError, StorageConfig};
use crate::config::tokenomics::{TokenomicsConfig, TokenomicsConfigError};

pub struct SharedConfig {
    genesis: Arc<GenesisConfig>,
    storage: Arc<StorageConfig>,
    tokenomics: Arc<TokenomicsConfig>,
}

pub struct SharedConfigError {
    pub genesis_config_error: Arc<GenesisConfigError>,
    pub storage_config_error: Arc<StorageConfigError>,
    pub tokenomics_config_error: Arc<TokenomicsConfigError>,
}

impl SharedConfig {
    pub fn new(
        genesis: GenesisConfig,
        storage: StorageConfig,
        tokenomics: TokenomicsConfig,
    ) -> Self {
        Self {
            genesis: Arc::new(genesis),
            storage: Arc::new(storage),
            tokenomics: Arc::new(tokenomics),
        }
    }

    pub fn load_default() -> Result<Arc<SharedConfig>, SharedConfigError> {
        let genesis = match GenesisConfig::load_default() {
            Ok(config) => config,
            Err(e) => return Err(SharedConfigError::from_genesis_error(e)),
        };

        let storage = match StorageConfig::load_default() {
            Ok(config) => config,
            Err(e) => return Err(SharedConfigError::from_storage_error(e)),
        };

        let tokenomics = match TokenomicsConfig::load_default() {
            Ok(config) => config,
            Err(e) => return Err(SharedConfigError::from_tokenomics_error(e)),
        };

        Ok(Arc::new(Self {
            genesis: Arc::new(genesis),
            storage: Arc::new(storage),
            tokenomics: Arc::new(tokenomics),
        }))
    }

    // Accessor methods to get references to the configurations
    pub fn genesis(&self) -> &GenesisConfig {
        &self.genesis
    }

    pub fn storage(&self) -> &StorageConfig {
        &self.storage
    }

    pub fn tokenomics(&self) -> &TokenomicsConfig {
        &self.tokenomics
    }
}

impl Clone for SharedConfig {
    fn clone(&self) -> Self {
        Self {
            genesis: Arc::clone(&self.genesis),
            storage: Arc::clone(&self.storage),
            tokenomics: Arc::clone(&self.tokenomics),
        }
    }
}

impl SharedConfigError {
    pub fn new(
        genesis_error: GenesisConfigError,
        storage_error: StorageConfigError,
        tokenomics_error: TokenomicsConfigError,
    ) -> Self {
        Self {
            genesis_config_error: Arc::new(genesis_error),
            storage_config_error: Arc::new(storage_error),
            tokenomics_config_error: Arc::new(tokenomics_error),
        }
    }

    // Helper methods for common error cases
    pub fn from_genesis_error(error: GenesisConfigError) -> Self {
        Self::new(
            error,
            StorageConfigError::ValidationError("Storage config not provided".to_string()),
            TokenomicsConfigError::ValidationError("Tokenomics config not provided".to_string()),
        )
    }

    pub fn from_storage_error(error: StorageConfigError) -> Self {
        Self::new(
            GenesisConfigError::ValidationError("Genesis config not provided".to_string()),
            error,
            TokenomicsConfigError::ValidationError("Tokenomics config not provided".to_string()),
        )
    }

    pub fn from_tokenomics_error(error: TokenomicsConfigError) -> Self {
        Self::new(
            GenesisConfigError::ValidationError("Genesis config not provided".to_string()),
            StorageConfigError::ValidationError("Storage config not provided".to_string()),
            error,
        )
    }
}

// Implement standard error handling
impl std::error::Error for SharedConfigError {}

impl std::fmt::Display for SharedConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Configuration error in shared configuration")
    }
}

impl std::fmt::Debug for SharedConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedConfigError")
            .field("genesis_error", &self.genesis_config_error)
            .field("storage_error", &self.storage_config_error)
            .field("tokenomics_error", &self.tokenomics_config_error)
            .finish()
    }
}
