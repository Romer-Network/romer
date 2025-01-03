use bytes::Bytes;
use commonware_runtime::Storage;
use commonware_storage::{
    archive::{Archive, Config as ArchiveConfig},
    journal::{Journal, Config as JournalConfig},
    metadata::{Metadata, Config as MetadataConfig},
};
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};
use prometheus_client::registry::Registry;

use crate::domain::block::entities::{Block, Transaction};
use crate::config::storage::StorageConfig;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Journal error: {0}")]
    Journal(String),
    #[error("Archive error: {0}")]
    Archive(String),
    #[error("Metadata error: {0}")]
    Metadata(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Initialization error: {0}")]
    Initialization(String),
}

/// Manages blockchain data persistence across different storage mechanisms
pub struct PersistenceManager<S: Storage<B>, B: commonware_runtime::Blob> {
    runtime: S,
    config: Arc<StorageConfig>,
    journal: Option<Journal<B, S>>,
    archive: Option<Archive<B, S>>,
    metadata: Option<Metadata<B, S>>,
    registry: Arc<prometheus_client::registry::Registry>,
}

/// Key-value pairs for metadata storage
#[derive(Serialize, Deserialize)]
struct BlockchainMetadata {
    latest_height: u64,
    latest_view: u32,
    genesis_hash: [u8; 32],
    network_version: String,
}

impl<S: Storage<B>, B: commonware_runtime::Blob> PersistenceManager<S, B> {
    pub fn new(runtime: S, config: Arc<StorageConfig>, registry: Arc<Registry>) -> Self {
        Self {
            runtime,
            config,
            journal: None,
            archive: None,
            metadata: None,
            registry,
        }
    }

    /// Initialize all storage components
    pub async fn initialize(&mut self) -> Result<(), PersistenceError> {
        info!("Initializing storage persistence layer");

        // Initialize metadata store for blockchain state
        let metadata_config = MetadataConfig {
            registry: Arc::clone(&self.registry),
            partition: self.config.metadata.validator_partition.clone(),
        };

        self.metadata = Some(
            Metadata::init(self.runtime.clone(), metadata_config)
                .await
                .map_err(|e| PersistenceError::Initialization(e.to_string()))?,
        );

        // Initialize journal for recent blocks
        let journal_config = JournalConfig {
            registry: Arc::clone(&self.registry),
            partition: self.config.journal.partitions.blocks.clone(),
        };

        self.journal = Some(
            Journal::init(self.runtime.clone(), journal_config)
                .await
                .map_err(|e| PersistenceError::Initialization(e.to_string()))?,
        );

        // Initialize archive for historical blocks
        let archive_config = ArchiveConfig {
            registry: Arc::clone(&self.registry),
            key_len: 32, // Block hash length
            translator: self.config.archive.translator.clone(),
            section_mask: self.config.archive.section_mask,
            pending_writes: self.config.archive.pending_writes,
            replay_concurrency: self.config.archive.replay_concurrency,
            compression: Some(self.config.archive.compression_level),
        };

        self.archive = Some(
            Archive::init(
                self.journal.as_ref().unwrap().clone(),
                archive_config,
            )
            .await
            .map_err(|e| PersistenceError::Initialization(e.to_string()))?,
        );

        info!("Storage persistence layer initialized successfully");
        Ok(())
    }

    /// Store a new block in the journal and update metadata
    pub async fn store_block(&mut self, block: &Block) -> Result<(), PersistenceError> {
        let serialized_block = bincode::serialize(block)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        // Store in journal first
        if let Some(journal) = &mut self.journal {
            journal
                .append(block.header.height, Bytes::from(serialized_block.clone()))
                .await
                .map_err(|e| PersistenceError::Journal(e.to_string()))?;
        } else {
            return Err(PersistenceError::Journal("Journal not initialized".to_string()));
        }

        // Update metadata
        if let Some(metadata) = &mut self.metadata {
            let blockchain_metadata = BlockchainMetadata {
                latest_height: block.header.height,
                latest_view: block.header.view,
                genesis_hash: if block.header.height == 0 {
                    let mut hash = [0u8; 32];
                    // Calculate genesis hash
                    hash
                } else {
                    // Get from current metadata
                    [0u8; 32] // Placeholder
                },
                network_version: self.config.network_version.clone(),
            };

            metadata
                .put(
                    0, // Key for blockchain metadata
                    bincode::serialize(&blockchain_metadata)
                        .map_err(|e| PersistenceError::Serialization(e.to_string()))?,
                )
                .map_err(|e| PersistenceError::Metadata(e.to_string()))?;

            metadata
                .sync()
                .await
                .map_err(|e| PersistenceError::Metadata(e.to_string()))?;
        }

        // Archive older blocks if needed
        if block.header.height > self.config.archive.block_threshold {
            self.archive_old_blocks().await?;
        }

        info!("Block {} stored successfully", block.header.height);
        Ok(())
    }

    /// Retrieve a block by height
    pub async fn get_block(&self, height: u64) -> Result<Option<Block>, PersistenceError> {
        // Try journal first for recent blocks
        if let Some(journal) = &self.journal {
            if let Ok(Some(data)) = journal.get(height, height, None).await {
                return Ok(Some(
                    bincode::deserialize(&data)
                        .map_err(|e| PersistenceError::Serialization(e.to_string()))?,
                ));
            }
        }

        // Fall back to archive for older blocks
        if let Some(archive) = &self.archive {
            // Implementation depends on your archive key strategy
            Ok(None) // Placeholder
        } else {
            Err(PersistenceError::Archive("Archive not initialized".to_string()))
        }
    }

    /// Archive blocks older than the configured threshold
    async fn archive_old_blocks(&mut self) -> Result<(), PersistenceError> {
        // Implementation for moving blocks from journal to archive
        Ok(()) // Placeholder
    }

    /// Clean up and close all storage components
    pub async fn close(mut self) -> Result<(), PersistenceError> {
        if let Some(metadata) = self.metadata.take() {
            metadata
                .close()
                .await
                .map_err(|e| PersistenceError::Metadata(e.to_string()))?;
        }

        if let Some(journal) = self.journal.take() {
            journal
                .close()
                .await
                .map_err(|e| PersistenceError::Journal(e.to_string()))?;
        }

        if let Some(archive) = self.archive.take() {
            archive
                .close()
                .await
                .map_err(|e| PersistenceError::Archive(e.to_string()))?;
        }

        info!("Storage persistence layer closed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add tests for block storage and retrieval
}