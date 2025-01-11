// src/fix/parser.rs

use super::types::*;
use fefix::prelude::*;
use fefix::tagvalue::{Config, Decoder, DecoderBuffered, SetGetField};

/// Handles parsing and initial validation of FIX messages
pub struct FixParser {
    config: FixConfig,
}

impl FixParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        Self {
            config: FixConfig::default(),
        }
    }

    /// Create a parser with custom configuration
    pub fn with_config(config: FixConfig) -> Self {
        Self { config }
    }

    /// Parse and validate a raw FIX message
    pub fn parse(&self, raw_message: &[u8]) -> FixResult<ValidatedMessage> {
        // Check message size
        if raw_message.len() > self.config.max_message_size {
            return Err(FixError::MessageTooLarge);
        }

        // Create decoder for the message
        let mut decoder = Decoder::new(self.config.dictionary.clone());
        
        // Decode the message
        let message = decoder.decode(raw_message)
            .map_err(FixError::ParseError)?;

        // Validate FIX version
        let begin_string = message.get_field::<BeginString>()
            .map_err(|_| FixError::MissingField("BeginString".to_string()))?;
            
        if begin_string.as_str() != self.config.required_version {
            return Err(FixError::InvalidVersion);
        }

        // Extract and validate message type
        let msg_type_raw = message.get_field::<MsgType>()
            .map_err(|_| FixError::MissingField("MsgType".to_string()))?;
            
        let msg_type = MessageType::from_fix(msg_type_raw.as_str().chars().next().unwrap())
            .ok_or_else(|| FixError::InvalidMessageType(msg_type_raw.as_str().to_string()))?;

        // Extract required header fields
        let sender_comp_id = message.get_field::<SenderCompID>()
            .map_err(|_| FixError::MissingField("SenderCompID".to_string()))?
            .as_str()
            .to_string();
            
        let target_comp_id = message.get_field::<TargetCompID>()
            .map_err(|_| FixError::MissingField("TargetCompID".to_string()))?
            .as_str()
            .to_string();
            
        let msg_seq_num = message.get_field::<MsgSeqNum>()
            .map_err(|_| FixError::MissingField("MsgSeqNum".to_string()))?
            .as_str()
            .parse::<u64>()
            .map_err(|_| FixError::InvalidFormat("Invalid MsgSeqNum".to_string()))?;

        // Create validated message
        Ok(ValidatedMessage {
            msg_type,
            message,
            sender_comp_id,
            target_comp_id,
            msg_seq_num,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_logon() {
        let parser = FixParser::new();
        
        // Create a valid FIX 4.2 logon message
        let logon_msg = b"8=FIX.4.2\x019=76\x0135=A\x0134=1\x0149=SENDER\x0156=TARGET\x0152=20240111-12:00:00\x0198=0\x01108=30\x0110=205\x01";
        
        let result = parser.parse(logon_msg);
        assert!(result.is_ok());
        
        let validated = result.unwrap();
        assert_eq!(validated.msg_type, MessageType::Logon);
        assert_eq!(validated.sender_comp_id, "SENDER");
        assert_eq!(validated.target_comp_id, "TARGET");
        assert_eq!(validated.msg_seq_num, 1);
    }

    #[test]
    fn test_invalid_version() {
        let parser = FixParser::new();
        
        // Create a FIX 4.1 message
        let msg = b"8=FIX.4.1\x019=76\x0135=A\x0134=1\x0149=SENDER\x0156=TARGET\x0152=20240111-12:00:00\x0198=0\x01108=30\x0110=205\x01";
        
        let result = parser.parse(msg);
        assert!(matches!(result, Err(FixError::InvalidVersion)));
    }

    #[test]
    fn test_message_too_large() {
        let parser = FixParser::new();
        
        // Create a message larger than max_message_size
        let large_msg = vec![b'1'; 5000];
        
        let result = parser.parse(&large_msg);
        assert!(matches!(result, Err(FixError::MessageTooLarge)));
    }
}