use serde::{Deserialize, Serialize}; 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub view: u32,
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: [u8; 32],
    pub transactions_root: [u8; 32],
    pub state_root: [u8; 32],
    pub validator_public_key: [u8; 32]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub from: [u8; 32],
    pub nonce: u64,
    pub gas_amount: u64,
    pub signature: [u8 ; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    TokenTransfer {
        to: [u8; 32],
        amount: u64,
        transfer_type: TransferType,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    Normal,
    Mint, 
    Burn,
}