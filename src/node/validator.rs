use commonware_cryptography::Ed25519;
use commonware_runtime::deterministic::Context as RuntimeContext;
use std::net::SocketAddr;
use thiserror::Error;
use tracing::{error, info};

use crate::config::genesis::ConfigError as GenesisConfigError;
use crate::config::genesis::GenesisConfig;
use crate::config::storage::ConfigError as StorageConfigError;
use crate::config::storage::StorageConfig;
use crate::config::validator::ValidatorConfig;
use crate::consensus::automaton::BlockchainAutomaton;
use crate::node::hardware_validator::HardwareDetector;
use crate::node::hardware_validator::OperatingSystem;
use crate::node::hardware_validator::VirtualizationType;
use crate::node::operating_regions::RegionConfig;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Genesis configuration error: {0}")]
    Genesis(#[from] GenesisConfigError),

    #[error("Storage configuration error: {0}")]
    Storage(#[from] StorageConfigError),

    #[error("Node initialization error: {0}")]
    Initialization(String),
}

/// The main Node structure that coordinates all components
pub struct Node {
    runtime: RuntimeContext,
    genesis_config: GenesisConfig,
    storage_config: StorageConfig,
    signer: Ed25519,
}

impl Node {
    /// Creates a new Node instance with validated configurations
    pub fn new(runtime: RuntimeContext, signer: Ed25519) -> Result<Self, NodeError> {
        let (genesis_config, storage_config) = Self::configure_node_context()?;

        Ok(Self {
            runtime,
            genesis_config,
            storage_config,
            signer,
        })
    }

    /// Loads and validates all required node configurations
    /// Returns a tuple of validated configurations or a NodeError if anything fails
    fn configure_node_context() -> Result<(GenesisConfig, StorageConfig), NodeError> {
        // Detect virtualization
        let virtualization_type = match HardwareDetector::detect_virtualization() {
            Ok(virt_type) => virt_type,
            Err(e) => {
                error!("Virtualization detection failed: {}", e);
                return Err(NodeError::Initialization(
                    "Failed to detect virtualization environment".to_string(),
                ));
            }
        };

        // Stop the program if not on physical hardware
        match virtualization_type {
            VirtualizationType::Physical => {
                info!("Running on physical hardware");
            }
            VirtualizationType::Virtual(tech) => {
                error!("Node detected running in virtual environment: {}", tech);
                return Err(NodeError::Initialization(format!(
                    "Node is not allowed to run in virtual environment: {}",
                    tech
                )));
            }
        }

        let genesis_config = GenesisConfig::load_default().map(|config| {
            info!("Genesis configuration loaded successfully");
            info!("Chain ID: {}", config.network.chain_id);
            config
        })?;

        // Load Storage configuration
        let storage_config = StorageConfig::load_default().map(|config| {
            info!("Storage configuration loaded successfully");
            config
        })?;

        Ok((genesis_config, storage_config))
    }

    pub async fn run(
        &self,
        address: SocketAddr,
        bootstrap: Option<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting node at {}", address);

        let automaton = BlockchainAutomaton::new(
            self.runtime.clone(),
            self.signer.clone(),
            self.genesis_config.clone(),
            self.storage_config.clone(),
        );

        automaton.run().await?;

        Ok(())
    }
}
