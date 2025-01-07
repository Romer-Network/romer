// src/node/node.rs

use anyhow::{Context, Result};
use commonware_cryptography::Ed25519;
use commonware_runtime::tokio::Context as RuntimeContext;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::node::keystore::keymanager::NodeKeyManager;

/// Represents a fully configured Rømer Chain node
pub struct Node {
    runtime: RuntimeContext,
    identity: Ed25519,
    storage_path: PathBuf,
    network_address: SocketAddr,
}

/// Builder for constructing a Node instance with all required configuration
pub struct NodeBuilder {
    runtime: Option<RuntimeContext>,
    node_id: Option<u64>,
    network_address: Option<SocketAddr>,
    identity: Option<Ed25519>,
    bootstrappers: Option<Vec<(Ed25519, SocketAddr)>>,
    participants: Option<Vec<u64>>,
    storage_path: Option<PathBuf>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            runtime: None,
            node_id: None,
            network_address: None,
            identity: None,
            bootstrappers: None,
            participants: None,
            storage_path: None,
        }
    }

    pub fn with_participants(mut self, participants: Vec<u64>) -> Self {
        // Store the participant IDs for network configuration
        self.participants = Some(participants);
        self
    }
    
    /// Set the node's runtime context
    pub fn with_runtime(mut self, runtime: RuntimeContext) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Configure node identity using Ed25519 keys
    pub fn with_identity(mut self, identity: Ed25519) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Set the storage path for node data
    pub fn with_storage_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.storage_path = Some(path.into());
        self
    }

    /// Configure the network address for P2P communication
    pub fn with_network_address(mut self, addr: SocketAddr) -> Self {
        self.network_address = Some(addr);
        self
    }

    /// Build the final Node instance, validating all required components
    pub fn build(self) -> Result<Node> {
        let runtime = self.runtime.context("Runtime context is required")?;
        let node_id = self.node_id.context("Node ID is required")?;
        let network_address = self.network_address.context("Network address is required")?;
        let identity = self.identity.context("Node identity is required")?;
        let participants = self.participants.context("Participant list is required")?;
        let storage_path = self.storage_path.context("Storage path is required")?;

        Ok(Node {
            runtime,
            identity,
            storage_path,
            network_address,
        })
    }
}

impl Node {
    /// Start the node and begin participating in the network
    pub async fn run(&self) -> Result<()> {
        // We'll implement this as we add more functionality
        Ok(())
    }
}
