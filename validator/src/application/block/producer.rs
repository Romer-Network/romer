/* 
use std::sync::Arc;
use thiserror::Error;
use commonware_cryptography::Ed25519;
use commonware_runtime::{Clock, SystemTimeExt};
use tracing::{info, warn};

use crate::config::shared::SharedConfig;
use crate::block::{
    entities::{Block, BlockHeader, Transaction, TransactionType, TransferType},
    state::BlockchainState,
    validator::BlockValidator,
};
use crate::utils::utils::BlockHasher;

#[derive(Error, Debug)]
pub enum BlockProductionError {
    #[error("Block creation failed: {0}")]
    Creation(String),
    #[error("State transition error: {0}")]
    StateTransition(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

// Domain events emitted by the BlockProducer
#[derive(Debug, Clone)]
pub enum BlockEvent {
    GenesisCreated(Block),
    BlockCreated(Block),
    BlockValidated(Block),
    ValidationFailed { reason: String },
}

pub struct BlockProducer {
    signer: Ed25519,
    config: Arc<SharedConfig>,
    state: BlockchainState,
    validator: BlockValidator,
    block_hasher: BlockHasher,
}

impl BlockProducer {
    pub fn new(
        signer: Ed25519,
        config: Arc<SharedConfig>,
        state: BlockchainState,
    ) -> Self {
        Self {
            signer,
            config,
            state,
            validator: BlockValidator::new(),
            block_hasher: BlockHasher::new(),
        }
    }

    /// Creates the genesis block with initial token distribution
    pub async fn create_genesis_block(&mut self) -> Result<BlockEvent, BlockProductionError> {
        info!("Creating genesis block");
        
        // Convert treasury address and prepare initial transaction
        let treasury_vec = self.block_hasher.address_to_bytes(
            &self.config.tokenomics().addresses.treasury
        );
        
        let mut treasury_bytes = [0u8; 32];
        treasury_bytes[..treasury_vec.len().min(32)]
            .copy_from_slice(&treasury_vec[..treasury_vec.len().min(32)]);

        // Create the genesis mint transaction
        let mint_transaction = Transaction {
            transaction_type: TransactionType::TokenTransfer {
                to: treasury_bytes,
                amount: self.config.tokenomics().supply.initial_supply,
                transfer_type: TransferType::Mint,
            },
            from: [0u8; 32],
            nonce: 0,
            gas_amount: 0,
            signature: [0u8; 32],
        };

        // Calculate roots
        let transactions_root = self.block_hasher
            .calculate_transactions_root(&[mint_transaction.clone()]);

        let initial_state = vec![(
            treasury_vec,
            self.config.tokenomics().supply.initial_supply,
        )];
        let state_root = self.block_hasher.calculate_state_root(&initial_state);

        // Prepare validator key
        let mut validator_key = [0u8; 32];
        validator_key.copy_from_slice(&self.signer.public_key());

        // Create the genesis block
        let block = Block {
            header: BlockHeader {
                view: 0,
                height: 0,
                timestamp: self.config.genesis().network.genesis_time,
                previous_hash: [0u8; 32],
                transactions_root,
                state_root,
                validator_public_key: validator_key,
            },
            transactions: vec![mint_transaction],
        };

        // Validate the genesis block
        if let Err(e) = self.validator.validate_genesis_block(&block) {
            return Err(BlockProductionError::Validation(e.to_string()));
        }

        // Apply state changes
        if let Err(e) = self.state.apply_genesis_block(&block) {
            return Err(BlockProductionError::StateTransition(e.to_string()));
        }

        info!("Genesis block created successfully");
        Ok(BlockEvent::GenesisCreated(block))
    }

    /// Creates a new block with pending transactions
    pub async fn create_block(
        &mut self,
        view: u32,
        transactions: Vec<Transaction>,
    ) -> Result<BlockEvent, BlockProductionError> {
        let previous_block = self.state.get_latest_block()
            .ok_or_else(|| BlockProductionError::Creation("No previous block found".to_string()))?;

        let transactions_root = self.block_hasher.calculate_transactions_root(&transactions);
        let state_root = self.calculate_new_state_root(&transactions)?;

        let mut validator_key = [0u8; 32];
        validator_key.copy_from_slice(&self.signer.public_key());

        let block = Block {
            header: BlockHeader {
                view,
                height: previous_block.header.height + 1,
                timestamp: SystemTime::now().unix_timestamp() as u64,
                previous_hash: self.block_hasher.calculate_block_hash(&previous_block),
                transactions_root,
                state_root,
                validator_public_key: validator_key,
            },
            transactions,
        };

        // Validate the block
        if let Err(e) = self.validator.validate_block(&block, &previous_block) {
            return Err(BlockProductionError::Validation(e.to_string()));
        }

        Ok(BlockEvent::BlockCreated(block))
    }

    /// Validates a block received from the network
    pub async fn validate_block(&self, block: &Block) -> Result<BlockEvent, BlockProductionError> {
        let previous_block = self.state.get_block_at_height(block.header.height - 1)
            .ok_or_else(|| BlockProductionError::Validation("Previous block not found".to_string()))?;

        if let Err(e) = self.validator.validate_block(block, &previous_block) {
            warn!("Block validation failed: {}", e);
            return Ok(BlockEvent::ValidationFailed { reason: e.to_string() });
        }

        Ok(BlockEvent::BlockValidated(block.clone()))
    }

    // Helper method to calculate new state root after applying transactions
    fn calculate_new_state_root(
        &self,
        transactions: &[Transaction]
    ) -> Result<[u8; 32], BlockProductionError> {
        // This would normally involve applying transactions to current state
        // and calculating new state root. Simplified for demonstration.
        let mut state_updates = Vec::new();
        
        for tx in transactions {
            if let TransactionType::TokenTransfer { to, amount, .. } = tx.transaction_type {
                state_updates.push((to.to_vec(), amount));
            }
        }

        Ok(self.block_hasher.calculate_state_root(&state_updates))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Add tests for block creation and validation
}
    */