use commonware_cryptography::Ed25519;
use commonware_runtime::deterministic::Context as RuntimeContext;
use futures::future::Shared;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

use crate::config::shared::{SharedConfig, SharedConfigError};
use crate::consensus::automaton::BlockchainAutomaton;
use crate::node::hardware_validator::HardwareDetector;
use crate::node::hardware_validator::VirtualizationType;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Configuration error")]
    Configuration(SharedConfigError),
    #[error("Node initialization error: {0}")]
    Initialization(String),
}

/// The main Node structure that coordinates all components
pub struct Node {
    runtime: RuntimeContext,
    config: Arc<SharedConfig>,
    signer: Ed25519,
}

impl Node {
    /// Creates a new Node instance with provided runtime, configuration, and signer
    pub fn new(
        runtime: RuntimeContext,
        config: Arc<SharedConfig>,
        signer: Ed25519,
    ) -> Result<Self, NodeError> {
        // Verify hardware requirements before proceeding
        Self::verify_hardware_requirements()?;

        Ok(Self {
            runtime,
            config,
            signer,
        })
    }

    /// Verifies the physical hardware requirements for running a node
    fn verify_hardware_requirements() -> Result<(), NodeError> {
        let virtualization_type = HardwareDetector::detect_virtualization().map_err(|e| {
            NodeError::Initialization(format!(
                "Failed to detect virtualization environment: {}",
                e
            ))
        })?;

        match virtualization_type {
            VirtualizationType::Physical => {
                info!("Running on physical hardware");
                Ok(())
            }
            VirtualizationType::Virtual(tech) => {
                error!("Node detected running in virtual environment: {}", tech);
                Err(NodeError::Initialization(format!(
                    "Node is not allowed to run in virtual environment: {}",
                    tech
                )))
            }
        }
    }

    /// Loads and validates all required node configurations and returns a SharedConfig instance
    fn configure_node_context() -> Result<Arc<SharedConfig>, NodeError> {
        // Load the shared configuration, converting any errors into NodeError
        SharedConfig::load_default().map_err(|e| {
            error!("Failed to load shared configuration: {}", e);
            NodeError::Configuration(e)
        })
    }

    pub async fn run(
        &self,
        address: SocketAddr,
        bootstrap: Option<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting node at {}", address);

        let mut automaton = BlockchainAutomaton::new(
            self.runtime.clone(),
            self.signer.clone(),
            self.config.clone(),
        );

        automaton.run(address, bootstrap).await?;

        Ok(())
    }
}
