use bytes::{BufMut, BytesMut};
use commonware_cryptography::{ Hasher, Sha256};

use crate::block::entities::{Block, Transaction, TransactionType, TransferType};

/// Provides core hashing functionality for the blockchain using Ed25519
#[derive(Clone)]
pub struct BlockHasher {
    hasher: Sha256,
}

impl BlockHasher {
    /// Creates a new BlockHasher instance
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Hash an entire block, producing a unique identifier
    /// This includes both the header and all transactions
    pub fn hash_block(&mut self, block: &Block) -> [u8; 32] {
        let mut buffer = BytesMut::new();

        // First hash the header
        buffer.put_u32_le(block.header.view);
        buffer.put_u64_le(block.header.height);

        // Convert SystemTime to nanoseconds since epoch
        buffer.put_u64_le(block.header.timestamp);

        // Add the three existing hashes
        buffer.put_slice(&block.header.previous_hash);
        buffer.put_slice(&block.header.transactions_root);
        buffer.put_slice(&block.header.state_root);
        buffer.put_slice(&block.header.validator_public_key);

        // Hash all fields together
        self.hasher.update(&buffer);
        let mut result = [0u8; 32];
        result.copy_from_slice(&self.hasher.finalize());
        self.hasher.reset();
        result
    }

    /// Calculate the Merkle root of all transactions in a block
    /// Uses a binary Merkle tree structure for efficient proofs
    pub fn calculate_transactions_root(&mut self, transactions: &[Transaction]) -> [u8; 32] {
        if transactions.is_empty() {
            return [0u8; 32];
        }

        // First, hash all individual transactions
        let mut hashes: Vec<[u8; 32]> = transactions
            .iter()
            .map(|tx| self.hash_transaction(tx))
            .collect();

        // Build the Merkle tree level by level
        while hashes.len() > 1 {
            let mut next_level = Vec::with_capacity((hashes.len() + 1) / 2);

            // Process pairs of hashes
            for chunk in hashes.chunks(2) {
                let mut buffer = BytesMut::new();
                buffer.put_slice(&chunk[0]);

                // If there's no second hash, duplicate the first one
                buffer.put_slice(chunk.get(1).unwrap_or(&chunk[0]));

                self.hasher.update(&buffer);
                let mut result = [0u8; 32];
                result.copy_from_slice(&self.hasher.finalize());
                self.hasher.reset();
                next_level.push(result);
            }

            hashes = next_level;
        }

        hashes[0]
    }

    /// Hash a single transaction deterministically
    pub fn hash_transaction(&mut self, transaction: &Transaction) -> [u8; 32] {
        let mut buffer = BytesMut::new();

        // Hash transaction type
        match &transaction.transaction_type {
            TransactionType::TokenTransfer {
                to,
                amount,
                transfer_type,
            } => {
                // Write discriminant for TokenTransfer (0 for first enum variant)
                buffer.put_u8(0);
                buffer.put_slice(to);
                buffer.put_u64_le(*amount);

                // Encode transfer type
                let transfer_type_value = match transfer_type {
                    TransferType::Normal => 0u8,
                    TransferType::Mint => 1u8,
                    TransferType::Burn => 2u8,
                };
                buffer.put_u8(transfer_type_value);
            }
        }

        // Add remaining transaction fields
        buffer.put_slice(&transaction.from);
        buffer.put_u64_le(transaction.nonce);
        buffer.put_u64_le(transaction.gas_amount);
        buffer.put_slice(&transaction.signature);

        self.hasher.update(&buffer);
        let mut result = [0u8; 32];
        result.copy_from_slice(&self.hasher.finalize());
        self.hasher.reset();
        result
    }

    pub fn address_to_bytes(&self, address: &str) -> Vec<u8> {
        // If the address starts with "0x", remove it
        let clean_address = address.trim_start_matches("0x");

        // Try to decode from hex first
        if let Ok(bytes) = hex::decode(clean_address) {
            return bytes;
        }

        // If not hex, fall back to raw bytes
        // In production, you might want to handle this case differently
        address.as_bytes().to_vec()
    }
    /// Calculate state root from a set of address/balance pairs
    /// Uses a simple concatenation for now - could be upgraded to a Merkle Patricia Trie
    pub fn calculate_state_root(&mut self, state_pairs: &[(Vec<u8>, u64)]) -> [u8; 32] {
        let mut buffer = BytesMut::new();

        // Sort pairs by address to ensure deterministic ordering
        let mut pairs = state_pairs.to_vec();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        // Hash each address/balance pair
        for (address, balance) in pairs {
            buffer.put_slice(&address);
            buffer.put_u64_le(balance);
        }

        self.hasher.update(&buffer);
        let mut result = [0u8; 32];
        result.copy_from_slice(&self.hasher.finalize());
        self.hasher.reset();
        result
    }
}

