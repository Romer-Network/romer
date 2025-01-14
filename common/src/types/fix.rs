use chrono::{DateTime, Utc};
use fefix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration settings for FIX protocol handling across the system.
/// We store the dictionary version rather than the Dictionary itself
/// since the fefix Dictionary type doesn't implement Serialize/Deserialize.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixConfig {
    /// The FIX protocol version to use (e.g., "4.2", "4.4")
    pub fix_version: String,

    /// The identifier of the message sender (SenderCompID in FIX)
    pub sender_comp_id: String,

    /// The identifier of the message recipient (TargetCompID in FIX)
    pub target_comp_id: String,
}

impl FixConfig {
    /// Gets the FIX dictionary for the configured version
    pub fn dictionary(&self) -> Dictionary {
        match self.fix_version.as_str() {
            "4.2" => Dictionary::fix42(),
            "4.4" => Dictionary::fix44(),
            // Default to FIX 4.2 for unknown versions
            _ => Dictionary::fix42(),
        }
    }
}

impl Default for FixConfig {
    fn default() -> Self {
        Self {
            fix_version: "4.2".to_string(),
            sender_comp_id: "SENDER".to_string(),
            target_comp_id: "Rømer".to_string(),
        }
    }
}

/// Represents the different types of FIX messages supported by the system.
/// This enum makes message type handling type-safe and explicit throughout the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Logon message (35=A) - Initiates a FIX session
    Logon,
    /// Logout message (35=5) - Terminates a FIX session
    Logout,
    /// Heartbeat message (35=0) - Keeps session alive
    Heartbeat,
    /// New Order Single message (35=D) - Submits a new order
    NewOrderSingle,
    /// Market Data Request message (35=V) - Requests market data
    MarketDataRequest,
    /// Market Data Snapshot message (35=W) - Provides market data
    MarketDataSnapshot,
}

impl MessageType {
    /// Converts a FIX message type value to our internal enum representation
    pub fn from_fix(msg_type: &str) -> Option<Self> {
        match msg_type {
            "A" => Some(Self::Logon),
            "5" => Some(Self::Logout),
            "0" => Some(Self::Heartbeat),
            "D" => Some(Self::NewOrderSingle),
            "V" => Some(Self::MarketDataRequest),
            "W" => Some(Self::MarketDataSnapshot),
            _ => None,
        }
    }

    /// Converts our internal enum representation to a FIX message type value
    pub fn to_fix(&self) -> &'static str {
        match self {
            Self::Logon => "A",
            Self::Logout => "5",
            Self::Heartbeat => "0",
            Self::NewOrderSingle => "D",
            Self::MarketDataRequest => "V",
            Self::MarketDataSnapshot => "W",
        }
    }
}

/// Represents a fully validated FIX protocol message.
/// This struct is used throughout the system to ensure consistent
/// message handling and validation across all components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedMessage {
    /// The type of FIX message, using our internal enum representation
    pub msg_type: MessageType,
    
    /// The message sender's identifier
    pub sender_comp_id: String,
    
    /// The message recipient's identifier
    pub target_comp_id: String,
    
    /// The message sequence number, used for gap detection and recovery
    pub msg_seq_num: u32,
    
    /// The complete message in raw byte form, including all fields and delimiters
    pub raw_data: Vec<u8>,
}

/// Common utility functions for FIX message handling
pub mod utils {
    use super::*;

    /// Generates a timestamp string in FIX protocol format (YYYYMMDD-HH:MM:SS).
    /// All timestamps in the system are in UTC to ensure consistency across regions.
    pub fn generate_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y%m%d-%H:%M:%S").to_string()
    }

    /// Calculates the FIX message checksum according to protocol specifications.
    /// The checksum is simply the sum of all bytes modulo 256, formatted as a
    /// three-digit string with leading zeros.
    pub fn calculate_checksum(msg: &[u8]) -> String {
        let sum: u32 = msg.iter().map(|&b| b as u32).sum();
        format!("{:03}", sum % 256)
    }

    /// Parses a raw FIX message into a map of field tags to values.
    /// This is useful for debugging and logging purposes.
    pub fn parse_message_fields(raw_data: &[u8]) -> HashMap<u32, String> {
        let mut fields = HashMap::new();
        let data = String::from_utf8_lossy(raw_data);
        
        for field in data.split('|') {
            if let Some((tag, value)) = field.split_once('=') {
                if let Ok(tag_num) = tag.parse::<u32>() {
                    fields.insert(tag_num, value.to_string());
                }
            }
        }
        
        fields
    }
}

/// Error types that can occur during FIX message processing
#[derive(Debug, thiserror::Error)]
pub enum FixError {
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
    
    #[error("Missing required field: {0}")]
    MissingField(u32),
    
    #[error("Invalid field value: {field} = {value}")]
    InvalidFieldValue {
        field: u32,
        value: String,
    },
    
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::from_fix("A"), Some(MessageType::Logon));
        assert_eq!(MessageType::Logon.to_fix(), "A");
    }

    #[test]
    fn test_checksum_calculation() {
        let msg = b"8=FIX.4.2|9=0|35=A|";
        let checksum = utils::calculate_checksum(msg);
        assert_eq!(checksum.len(), 3);
    }
}