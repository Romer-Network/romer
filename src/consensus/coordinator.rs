use commonware_consensus::{simplex::Context, Committer, Relay, Supervisor};
use commonware_consensus::simplex::{Config as SimplexConfig, Engine};
use commonware_cryptography::{Ed25519, PublicKey};
use commonware_p2p::{Recipients, Sender};
use commonware_runtime::deterministic::Context as RuntimeContext;
use commonware_storage::journal::{Journal, Config as JournalConfig};
use bytes::{Bytes, BytesMut};
use futures::channel::oneshot;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

use crate::config::shared::SharedConfig;
use crate::domain::block::{
    producer::BlockProducer,
    entities::Block,
    state::BlockchainState
};
use crate::consensus::supervisor::BlockchainSupervisor;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Block production failed: {0}")]
    BlockProduction(String),
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("Network error: {0}")]
    Network(String),
}

pub struct ConsensusCoordinator {
    runtime: RuntimeContext,
    config: Arc<SharedConfig>,
    signer: Ed25519,
    block_producer: BlockProducer,
    state: Arc<BlockchainState>,
    supervisor: BlockchainSupervisor,
    p2p_sender: Option<Box<dyn Sender>>,
}

impl ConsensusCoordinator {
    pub fn new(
        runtime: RuntimeContext,
        config: Arc<SharedConfig>,
        signer: Ed25519,
        state: Arc<BlockchainState>,
    ) -> Self {
        let block_producer = BlockProducer::new(
            signer.clone(),
            Arc::clone(&config),
            (*state).clone(),
        );

        let supervisor = BlockchainSupervisor::new(signer.public_key());

        Self {
            runtime,
            config,
            signer,
            block_producer,
            state,
            supervisor,
            p2p_sender: None,
        }
    }

    pub fn set_network_sender(&mut self, sender: Box<dyn Sender>) {
        self.p2p_sender = Some(sender);
    }

    pub async fn start_consensus(
        &mut self,
        journal: Journal,
    ) -> Result<(), ConsensusError> {
        info!("Initializing consensus mechanism");

        // Configure Simplex consensus
        let consensus_config = SimplexConfig {
            namespace: self.config.genesis().network.chain_id.clone().into_bytes(),
            mailbox_size: self.config.genesis().consensus.max_message_size,
            leader_timeout: self.config.genesis().consensus.leader_timeout,
            notarization_timeout: self.config.genesis().consensus.notarization_timeout,
            nullify_retry: self.config.genesis().consensus.nullify_retry,
            activity_timeout: self.config.genesis().consensus.activity_timeout,
            fetch_timeout: self.config.genesis().consensus.fetch_timeout,
            max_fetch_count: self.config.genesis().consensus.max_fetch_count,
            max_fetch_size: self.config.genesis().consensus.max_fetch_size,
            fetch_rate_per_peer: self.config.genesis().consensus.fetch_rate.clone(),
            fetch_concurrent: self.config.genesis().consensus.fetch_concurrent,
            ..Default::default()
        };

        // Initialize Simplex consensus engine
        let engine = Engine::new(
            self.runtime.clone(),
            journal,
            consensus_config,
        ).map_err(|e| ConsensusError::Consensus(e.to_string()))?;

        info!("Starting consensus engine");
        self.run_consensus(engine).await
    }

    async fn run_consensus(
        &mut self,
        engine: Engine,
    ) -> Result<(), ConsensusError> {
        // Create channels for consensus communication
        let (consensus_tx, consensus_rx) = futures::channel::mpsc::channel(100);
        let (network_tx, network_rx) = futures::channel::mpsc::channel(100);

        // Start the consensus engine
        engine.run((consensus_tx, consensus_rx), (network_tx, network_rx))
            .await
            .map_err(|e| ConsensusError::Consensus(e.to_string()))?;

        info!("Consensus engine started successfully");
        Ok(())
    }
}

// Implement consensus traits
impl Relay for ConsensusCoordinator {
    async fn broadcast(&mut self, payload: Bytes) {
        if let Some(sender) = &mut self.p2p_sender {
            if let Err(e) = sender.send(Recipients::All, payload, true).await {
                warn!("Failed to broadcast consensus message: {}", e);
            }
        }
    }
}

impl Committer for ConsensusCoordinator {
    async fn prepared(&mut self, proof: Bytes, payload: Bytes) {
        // Handle block preparation
        match bincode::deserialize::<Block>(&payload) {
            Ok(block) => {
                info!("Block prepared for consensus: height={}", block.header.height);
                // Additional preparation logic
            }
            Err(e) => warn!("Failed to deserialize prepared block: {}", e),
        }
    }

    async fn finalized(&mut self, proof: Bytes, payload: Bytes) {
        // Handle block finalization
        match bincode::deserialize::<Block>(&payload) {
            Ok(block) => {
                info!("Block finalized by consensus: height={}", block.header.height);
                if let Err(e) = self.state.apply_block(&block) {
                    warn!("Failed to apply finalized block: {}", e);
                }
            }
            Err(e) => warn!("Failed to deserialize finalized block: {}", e),
        }
    }
}

impl Supervisor for ConsensusCoordinator {
    type Index = u64;
    type Seed = ();

    fn leader(&self, index: Self::Index, _seed: Self::Seed) -> Option<PublicKey> {
        self.supervisor.get_leader(index)
    }

    fn participants(&self, index: Self::Index) -> Option<&Vec<PublicKey>> {
        self.supervisor.get_participants(index)
    }

    fn is_participant(
        &self,
        index: Self::Index,
        candidate: &PublicKey,
    ) -> Option<u32> {
        self.supervisor.get_participant_index(index, candidate)
    }

    async fn report(&self, activity: u8, proof: Bytes) {
        // Handle validator activity reports
        info!("Validator activity reported: type={}", activity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add tests for consensus coordination
}