// src/fix/types.rs

use fefix::prelude::*;
use fefix::tagvalue::{Config, Dictionary};
use thiserror::Error;

/// Represents the core message types we support in FIX 4.2
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    // Session messages
    Logon,              // Type = 'A'
    Logout,             // Type = '5'
    Heartbeat,          // Type = '0'
    TestRequest,        // Type = '1'
    ResendRequest,      // Type = '2'
    SequenceReset,      // Type = '4'
    
    // Application messages
    NewOrderSingle,     // Type = 'D'
    OrderCancelRequest, // Type = 'F'
    MarketDataRequest,  // Type = 'V'
}

impl MessageType {
    /// Convert a FIX message type char into our enum
    pub fn from_fix(typ: char) -> Option<Self> {
        match typ {
            'A' => Some(Self::Logon),
            '5' => Some(Self::Logout),
            '0' => Some(Self::Heartbeat),
            '1' => Some(Self::TestRequest),
            '2' => Some(Self::ResendRequest),
            '4' => Some(Self::SequenceReset),
            'D' => Some(Self::NewOrderSingle),
            'F' => Some(Self::OrderCancelRequest),
            'V' => Some(Self::MarketDataRequest),
            _ => None,
        }
    }
}

/// Core configuration for our FIX decoder/encoder
pub struct FixConfig {
    /// The FIX dictionary configuration
    dictionary: Dictionary,
    /// Maximum message size we'll accept
    max_message_size: usize,
    /// Required FIX version (4.2)
    required_version: String,
}

impl Default for FixConfig {
    fn default() -> Self {
        Self {
            dictionary: Dictionary::fix42(), // Use FIX 4.2 dictionary
            max_message_size: 4096,         // 4KB max message size
            required_version: "FIX.4.2".to_string(),
        }
    }
}

/// Represents a validated FIX message ready for processing
#[derive(Debug)]
pub struct ValidatedMessage {
    /// The type of message
    pub msg_type: MessageType,
    /// The raw FIX message for processing
    pub message: fefix::tagvalue::Message,
    /// Sender's comp ID
    pub sender_comp_id: String,
    /// Target comp ID
    pub target_comp_id: String,
    /// Message sequence number
    pub msg_seq_num: u64,
}

/// Errors that can occur during FIX message processing
#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
    
    #[error("Invalid FIX version (requires FIX.4.2)")]
    InvalidVersion,
    
    #[error("Message too large")]
    MessageTooLarge,
    
    #[error("Parsing error: {0}")]
    ParseError(#[from] fefix::Error),
}

/// Result type for FIX operations
pub type FixResult<T> = Result<T, FixError>;