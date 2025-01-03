/* 
use bytes::{Bytes, BytesMut};
use commonware_consensus::{simplex::Context, Automaton};
use commonware_cryptography::Ed25519;
use commonware_runtime::deterministic::Context as RuntimeContext;
use futures::channel::oneshot;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

use crate::config::shared::SharedConfig;
use crate::consensus::coordinator::ConsensusCoordinator;
use crate::block::{
    producer::BlockProducer,
    state::BlockchainState,
    entities::Block,
};
use crate::storage::persistence::PersistenceManager;

#[derive(Error, Debug)]
pub enum AutomatonError {
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("Block production error: {0}")]
    BlockProduction(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("State error: {0}")]
    State(String),
}

/// Core blockchain automaton that coordinates between major system components
pub struct BlockchainAutomaton<S, B> 
where 
    S: commonware_runtime::Storage<B>,
    B: commonware_runtime::Blob,
{
    runtime: RuntimeContext,
    config: Arc<SharedConfig>,
    signer: Ed25519,
    
    // Core components 
    consensus: ConsensusCoordinator,
    block_producer: BlockProducer,
    persistence: PersistenceManager<S, B>,
    
    // Shared state
    state: Arc<BlockchainState>,
}

impl<S, B> BlockchainAutomaton<S, B> 
where 
    S: commonware_runtime::Storage<B>,
    B: commonware_runtime::Blob,
{
    pub fn new(
        runtime: RuntimeContext,
        config: Arc<SharedConfig>,
        signer: Ed25519,
        storage: S,
    ) -> Result<Self, AutomatonError> {
        // Initialize shared state
        let state = Arc::new(BlockchainState::new());
        
        // Initialize core components
        let block_producer = BlockProducer::new(
            signer.clone(),
            Arc::clone(&config),
            (*state).clone(),
        );

        let consensus = ConsensusCoordinator::new(
            runtime.clone(),
            Arc::clone(&config),
            signer.clone(),
            Arc::clone(&state),
        );

        let persistence = PersistenceManager::new(
            storage,
            Arc::new(config.storage().clone()),
            Arc::new(prometheus_client::registry::Registry::default()),
        );

        Ok(Self {
            runtime,
            config,
            signer,
            consensus,
            block_producer,
            persistence,
            state,
        })
    }

    /// Initialize the automaton and its components
    pub async fn initialize(&mut self) -> Result<(), AutomatonError> {
        info!("Initializing blockchain automaton");

        // Initialize persistence layer first
        self.persistence.initialize().await
            .map_err(|e| AutomatonError::Storage(e.to_string()))?;

        // Create genesis block if needed
        if self.state.get_latest_block().is_none() {
            info!("No existing chain found, creating genesis block");
            self.initialize_genesis().await?;
        }

        info!("Blockchain automaton initialized successfully");
        Ok(())
    }

    /// Initialize the blockchain with genesis block
    async fn initialize_genesis(&mut self) -> Result<(), AutomatonError> {
        // Generate genesis block
        let genesis_event = self.block_producer.create_genesis_block()
            .await
            .map_err(|e| AutomatonError::BlockProduction(e.to_string()))?;

        // Extract the block from the event
        let genesis_block = match genesis_event {
            crate::domain::block::producer::BlockEvent::GenesisCreated(block) => block,
            _ => return Err(AutomatonError::BlockProduction(
                "Unexpected event type from genesis creation".to_string()
            )),
        };

        // Store genesis block
        self.persistence.store_block(&genesis_block)
            .await
            .map_err(|e| AutomatonError::Storage(e.to_string()))?;

        // Initialize state with genesis block
        self.state.apply_genesis_block(&genesis_block)
            .map_err(|e| AutomatonError::State(e.to_string()))?;

        info!("Genesis block created and stored successfully");
        Ok(())
    }

    /// Set the network sender for consensus communication
    pub fn set_network_sender(&mut self, sender: Box<dyn commonware_p2p::Sender>) {
        self.consensus.set_network_sender(sender);
    }
}

impl<S, B> Automaton for BlockchainAutomaton<S, B> 
where 
    S: commonware_runtime::Storage<B>,
    B: commonware_runtime::Blob,
{
    type Context = Context;

    async fn genesis(&mut self) -> Bytes {
        match self.state.get_block_at_height(0) {
            Some(genesis_block) => {
                // Serialize the existing genesis block
                let mut buffer = BytesMut::new();
                // Serialize block header fields
                buffer.freeze()
            }
            None => {
                warn!("Genesis block not found in state");
                Bytes::new()
            }
        }
    }

    async fn propose(&mut self, context: Self::Context) -> oneshot::Receiver<Bytes> {
        let (tx, rx) = oneshot::channel();
        
        // Create new block
        let result = self.block_producer.create_block(
            context.view,
            Vec::new(), // TODO: Get pending transactions
        ).await;

        match result {
            Ok(event) => {
                match event {
                    crate::domain::block::producer::BlockEvent::BlockCreated(block) => {
                        // Serialize the block
                        if let Ok(block_bytes) = bincode::serialize(&block) {
                            let _ = tx.send(Bytes::from(block_bytes));
                        }
                    }
                    _ => warn!("Unexpected block event type during proposal"),
                }
            }
            Err(e) => warn!("Failed to create block proposal: {}", e),
        }

        rx
    }

    async fn verify(&mut self, context: Self::Context, payload: Bytes) -> oneshot::Receiver<bool> {
        let (tx, rx) = oneshot::channel();

        match bincode::deserialize::<Block>(&payload) {
            Ok(block) => {
                // Verify the block
                let result = self.block_producer.validate_block(&block).await;
                match result {
                    Ok(event) => {
                        match event {
                            crate::domain::block::producer::BlockEvent::BlockValidated(_) => {
                                let _ = tx.send(true);
                            }
                            crate::domain::block::producer::BlockEvent::ValidationFailed { reason } => {
                                warn!("Block validation failed: {}", reason);
                                let _ = tx.send(false);
                            }
                            _ => {
                                warn!("Unexpected validation event type");
                                let _ = tx.send(false);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Block validation error: {}", e);
                        let _ = tx.send(false);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to deserialize block for verification: {}", e);
                let _ = tx.send(false);
            }
        }

        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add tests for automaton coordination
}
    */