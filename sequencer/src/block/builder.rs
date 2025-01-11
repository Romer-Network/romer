use crate::fix::types::ValidatedMessage;
use super::batch::MessageBatch;
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Represents a complete block ready for the builder service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header containing metadata
    pub header: BlockHeader,
    /// The FIX messages contained in this block
    pub messages: Vec<ValidatedMessage>,
    /// Hash of the block's contents
    pub block_hash: String,
}

/// Contains metadata about the block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Unique identifier for this block
    pub block_id: u64,
    /// Hash of the previous block
    pub previous_hash: String,
    /// When this block was created
    pub timestamp: DateTime<Utc>,
    /// Number of messages in the block
    pub message_count: usize,
    /// Merkle root of the messages
    pub messages_root: String,
    /// Sequence number from the batch
    pub batch_sequence: u64,
}

/// Responsible for constructing blocks from message batches
pub struct BlockBuilder {
    /// The hash of the most recent block
    previous_hash: String,
    /// The current block number
    current_block_id: u64,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            // Initialize with genesis block hash
            previous_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            current_block_id: 0,
        }
    }

    /// Build a new block from a batch of messages
    pub fn build_block(&mut self, batch: MessageBatch) -> Block {
        // Calculate the merkle root of messages
        let messages_root = self.calculate_messages_root(&batch.messages);

        // Create the block header
        let header = BlockHeader {
            block_id: self.current_block_id,
            previous_hash: self.previous_hash.clone(),
            timestamp: Utc::now(),
            message_count: batch.messages.len(),
            messages_root,
            batch_sequence: batch.sequence,
        };

        // Calculate block hash
        let block_hash = self.calculate_block_hash(&header);

        // Update builder state
        self.previous_hash = block_hash.clone();
        self.current_block_id += 1;

        // Construct and return the full block
        Block {
            header,
            messages: batch.messages,
            block_hash,
        }
    }

    /// Calculate the merkle root of the messages
    fn calculate_messages_root(&self, messages: &[ValidatedMessage]) -> String {
        // For now, we'll use a simple concatenated hash
        // In production, this should be a proper merkle tree
        let mut hasher = Sha256::new();
        
        for msg in messages {
            // Hash each message's key fields
            hasher.update(msg.sender_comp_id.as_bytes());
            hasher.update(msg.target_comp_id.as_bytes());
            hasher.update(&msg.msg_seq_num.to_le_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Calculate the hash of the block
    fn calculate_block_hash(&self, header: &BlockHeader) -> String {
        let mut hasher = Sha256::new();
        
        // Hash key header fields
        hasher.update(header.block_id.to_le_bytes());
        hasher.update(header.previous_hash.as_bytes());
        hasher.update(header.timestamp.timestamp().to_le_bytes());
        hasher.update(header.message_count.to_le_bytes());
        hasher.update(header.messages_root.as_bytes());
        hasher.update(header.batch_sequence.to_le_bytes());

        hex::encode(hasher.finalize())
    }

    /// Verify a block's integrity
    pub fn verify_block(&self, block: &Block) -> bool {
        // Verify the block hash
        let calculated_hash = self.calculate_block_hash(&block.header);
        if calculated_hash != block.block_hash {
            return false;
        }

        // Verify the messages root
        let calculated_root = self.calculate_messages_root(&block.messages);
        if calculated_root != block.header.messages_root {
            return false;
        }

        // Verify message count
        if block.messages.len() != block.header.message_count {
            return false;
        }

        true
    }
}

/// Configuration for the block builder
#[derive(Debug, Clone)]
pub struct BlockConfig {
    /// Maximum size of a block in bytes
    pub max_block_size: usize,
    /// Maximum number of messages per block
    pub max_messages: usize,
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self {
            max_block_size: 1024 * 1024, // 1MB
            max_messages: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::types::MessageType;

    fn create_test_message(seq: u64) -> ValidatedMessage {
        ValidatedMessage {
            msg_type: MessageType::NewOrderSingle,
            message: fefix::tagvalue::Message::new(fefix::Dictionary::fix42()),
            sender_comp_id: "SENDER".to_string(),
            target_comp_id: "TARGET".to_string(),
            msg_seq_num: seq,
        }
    }

    fn create_test_batch(sequence: u64, message_count: usize) -> MessageBatch {
        let messages = (0..message_count)
            .map(|i| create_test_message(i as u64))
            .collect();

        MessageBatch {
            messages,
            start_time: tokio::time::Instant::now(),
            end_time: tokio::time::Instant::now(),
            sequence,
        }
    }

    #[test]
    fn test_block_creation_and_verification() {
        let mut builder = BlockBuilder::new();
        
        // Create a test batch
        let batch = create_test_batch(0, 5);
        
        // Build a block
        let block = builder.build_block(batch);
        
        // Verify the block
        assert!(builder.verify_block(&block));
        assert_eq!(block.header.message_count, 5);
        assert_eq!(block.header.block_id, 0);
    }

    #[test]
    fn test_sequential_blocks() {
        let mut builder = BlockBuilder::new();
        
        // Create two sequential blocks
        let block1 = builder.build_block(create_test_batch(0, 3));
        let block2 = builder.build_block(create_test_batch(1, 3));
        
        // Verify sequential properties
        assert_eq!(block2.header.previous_hash, block1.block_hash);
        assert_eq!(block2.header.block_id, 1);
    }
}