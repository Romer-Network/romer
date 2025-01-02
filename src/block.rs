use commonware_cryptography::{PublicKey, Signature};
use std::time::SystemTime;

/// Represents the header portion of a block, containing metadata and cryptographic links
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub view: u32,                     // Consensus view number when block was created
    pub height: u64,                   // Block height in the chain
    pub timestamp: SystemTime,         // Block creation time
    pub previous_hash: [u8; 32],       // Hash of the previous block
    pub transactions_root: [u8; 32],   // Merkle root of transactions
    pub state_root: [u8; 32],          // Root hash of the state trie
    pub validator_public_key: PublicKey,// Public key of the block producer
    pub utilization: f64,              // Current utilization vs base threshold
}

/// A complete block containing a header and a list of transactions
#[derive(Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}


/// A transaction that can be included in a block
#[derive(Debug, Clone)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub from: String,              // Base58 encoded address
    pub nonce: u64,                // Transaction sequence number
    pub gas_amount: u64,           // Computed gas requirement
    pub signature: Signature,      // Transaction signature
}

/// The different types of transactions supported by the system
#[derive(Debug, Clone)]
pub enum TransactionType {
    TokenTransfer {
        to: String,                // Base58 encoded recipient
        amount: u64,               // Amount in smallest unit (8 decimals)
    }
}
