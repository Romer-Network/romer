/* 
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tracing::{info, warn};

use crate::block::entities::{Block, Transaction, TransactionType};

#[derive(Error, Debug)]
pub enum StateError {
    #[error("State transition failed: {0}")]
    TransitionFailed(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Block not found at height: {0}")]
    BlockNotFound(u64),
}

pub struct BlockchainState {
    // Using RwLock for concurrent access to state
    blocks: Arc<RwLock<HashMap<u64, Block>>>,
    balances: Arc<RwLock<HashMap<Vec<u8>, u64>>>,
    latest_height: Arc<RwLock<u64>>,
}

impl BlockchainState {
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            latest_height: Arc::new(RwLock::new(0)),
        }
    }

    /// Applies the genesis block to initialize the blockchain state
    pub fn apply_genesis_block(&self, block: &Block) -> Result<(), StateError> {
        if block.header.height != 0 {
            return Err(StateError::InvalidState(
                "Genesis block must have height 0".to_string(),
            ));
        }

        // Initialize state with genesis block
        let mut blocks = self.blocks.write().map_err(|_| {
            StateError::TransitionFailed("Failed to acquire blocks lock".to_string())
        })?;

        let mut balances = self.balances.write().map_err(|_| {
            StateError::TransitionFailed("Failed to acquire balances lock".to_string())
        })?;

        let mut latest_height = self.latest_height.write().map_err(|_| {
            StateError::TransitionFailed("Failed to acquire height lock".to_string())
        })?;

        // Process genesis transactions
        for tx in &block.transactions {
            if let TransactionType::TokenTransfer { to, amount, .. } = &tx.transaction_type {
                balances.insert(to.to_vec(), *amount);
            }
        }

        // Store the genesis block
        blocks.insert(0, block.clone());
        *latest_height = 0;

        info!("Genesis block applied successfully");
        Ok(())
    }

    /// Applies a new block to the current state
    pub fn apply_block(&self, block: &Block) -> Result<(), StateError> {
        // Verify block height is sequential
        let expected_height = {
            let height = self.latest_height.read().map_err(|_| {
                StateError::TransitionFailed("Failed to read height".to_string())
            })?;
            *height + 1
        };

        if block.header.height != expected_height {
            return Err(StateError::InvalidState(format!(
                "Block height {} is not sequential. Expected {}",
                block.header.height, expected_height
            )));
        }

        // Process all transactions and update state
        let mut balances = self.balances.write().map_err(|_| {
            StateError::TransitionFailed("Failed to acquire balances lock".to_string())
        })?;

        for tx in &block.transactions {
            self.process_transaction(tx, &mut balances)?;
        }

        // Store the new block
        let mut blocks = self.blocks.write().map_err(|_| {
            StateError::TransitionFailed("Failed to acquire blocks lock".to_string())
        })?;

        blocks.insert(block.header.height, block.clone());

        // Update latest height
        let mut latest_height = self.latest_height.write().map_err(|_| {
            StateError::TransitionFailed("Failed to acquire height lock".to_string())
        })?;
        *latest_height = block.header.height;

        info!("Block {} applied successfully", block.header.height);
        Ok(())
    }

    /// Process a single transaction and update balances
    fn process_transaction(
        &self,
        tx: &Transaction,
        balances: &mut HashMap<Vec<u8>, u64>,
    ) -> Result<(), StateError> {
        match &tx.transaction_type {
            TransactionType::TokenTransfer { to, amount, transfer_type } => {
                match transfer_type {
                    TransferType::Mint => {
                        // Add new tokens to recipient
                        let current_balance = balances.get(&to.to_vec()).unwrap_or(&0);
                        balances.insert(to.to_vec(), current_balance + amount);
                    }
                    TransferType::Burn => {
                        // Remove tokens from sender
                        let sender_balance = balances.get(&tx.from.to_vec()).unwrap_or(&0);
                        if *sender_balance < *amount {
                            return Err(StateError::TransitionFailed(
                                "Insufficient balance for burn".to_string(),
                            ));
                        }
                        balances.insert(tx.from.to_vec(), sender_balance - amount);
                    }
                    TransferType::Normal => {
                        // Regular transfer between accounts
                        let sender_balance = balances.get(&tx.from.to_vec()).unwrap_or(&0);
                        if *sender_balance < *amount {
                            return Err(StateError::TransitionFailed(
                                "Insufficient balance for transfer".to_string(),
                            ));
                        }
                        
                        let recipient_balance = balances.get(&to.to_vec()).unwrap_or(&0);
                        
                        balances.insert(tx.from.to_vec(), sender_balance - amount);
                        balances.insert(to.to_vec(), recipient_balance + amount);
                    }
                }
            }
        }
        Ok(())
    }

    /// Gets a block at a specific height
    pub fn get_block_at_height(&self, height: u64) -> Option<Block> {
        self.blocks
            .read()
            .ok()
            .and_then(|blocks| blocks.get(&height).cloned())
    }

    /// Gets the latest block
    pub fn get_latest_block(&self) -> Option<Block> {
        let height = *self.latest_height.read().ok()?;
        self.get_block_at_height(height)
    }

    /// Gets the balance for an account
    pub fn get_balance(&self, account: &[u8]) -> Result<u64, StateError> {
        let balances = self.balances.read().map_err(|_| {
            StateError::TransitionFailed("Failed to read balances".to_string())
        })?;
        Ok(*balances.get(account).unwrap_or(&0))
    }

    /// Gets the current blockchain height
    pub fn get_height(&self) -> Result<u64, StateError> {
        Ok(*self.latest_height.read().map_err(|_| {
            StateError::TransitionFailed("Failed to read height".to_string())
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Add state transition and balance tracking tests
}
    */