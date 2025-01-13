// src/fix/parser.rs
/*  
use super::types::*;
use fefix::tagvalue::{Config, Decoder, Message, FieldAccess};
use fefix::Dictionary;
use chrono::Utc;
use std::str;
use tracing::{debug, warn};

/// The FIX parser handles initial message validation and field extraction.
/// It ensures messages conform to the FIX 4.2 protocol structure before
/// they're processed by the business logic.
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
    /// Returns a ValidatedMessage containing the parsed fields and message type
    pub fn parse(&self, raw_message: &[u8]) -> FixResult<ValidatedMessage<'_, Vec<u8>>> {
        // Validate message size first
        if raw_message.len() > self.config.max_message_size {
            warn!("Message exceeds maximum size limit");
            return Err(FixError::MessageTooLarge);
        }

        // Create decoder with our FIX dictionary
        let mut decoder = Decoder::new(self.config.dictionary.clone());
        
        // Attempt to decode the raw message
        let message = decoder.decode(raw_message)
            .map_err(|e| {
                warn!("Failed to decode message: {}", e);
                FixError::ParseError(e)
            })?;

        // Validate FIX version (tag 8)
        let begin_string = message.fv_raw(&8)
            .ok_or_else(|| FixError::MissingField("BeginString".to_string()))?;
            
        if begin_string != self.config.required_version.as_bytes() {
            warn!("Invalid FIX version");
            return Err(FixError::InvalidVersion);
        }

        // Extract message type (tag 35)
        let msg_type_raw = message.fv_raw(&35)
            .ok_or_else(|| FixError::MissingField("MsgType".to_string()))?;
            
        let msg_type = MessageType::from_fix(
            str::from_utf8(msg_type_raw)
                .map_err(|_| FixError::InvalidFormat("Invalid MsgType encoding".to_string()))?
                .chars()
                .next()
                .ok_or_else(|| FixError::InvalidFormat("Empty MsgType".to_string()))?
        ).ok_or_else(|| FixError::InvalidMessageType(
            String::from_utf8_lossy(msg_type_raw).to_string()
        ))?;

        // Extract sender comp ID (tag 49)
        let sender_comp_id = self.extract_string_field(&message, 49, "SenderCompID")?;

        // Extract target comp ID (tag 56)
        let target_comp_id = self.extract_string_field(&message, 56, "TargetCompID")?;

        // Extract message sequence number (tag 34)
        let msg_seq_num = self.extract_numeric_field::<u64>(&message, 34, "MsgSeqNum")?;

        // Extract sending time (tag 52) if present
        if let Some(sending_time) = message.fv_raw(&52) {
            // Validate sending time format
            if !self.validate_timestamp(sending_time) {
                return Err(FixError::InvalidFormat("Invalid SendingTime format".to_string()));
            }
        }

        debug!(
            msg_type = ?msg_type,
            sender = %sender_comp_id,
            target = %target_comp_id,
            seq = msg_seq_num,
            "Successfully parsed FIX message"
        );

        Ok(ValidatedMessage {
            msg_type,
            message,
            sender_comp_id,
            target_comp_id,
            msg_seq_num,
        })
    }

    /// Helper method to extract and convert a string field
    fn extract_string_field(&self, message: &Message<&[u8]>, tag: u32, field_name: &str) -> FixResult<String> {
        let field_value = message.fv_raw(&tag)
            .ok_or_else(|| FixError::MissingField(field_name.to_string()))?;
            
        String::from_utf8(field_value.to_vec())
            .map_err(|_| FixError::InvalidFormat(format!("Invalid {} encoding", field_name)))
    }

    /// Helper method to extract and convert a numeric field
    fn extract_numeric_field<T>(&self, message: &Message<&[u8]>, tag: u32, field_name: &str) -> FixResult<T> 
    where 
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let field_value = message.fv_raw(&tag)
            .ok_or_else(|| FixError::MissingField(field_name.to_string()))?;
            
        str::from_utf8(field_value)
            .map_err(|_| FixError::InvalidFormat(format!("Invalid {} encoding", field_name)))?
            .parse::<T>()
            .map_err(|e| FixError::InvalidFormat(format!("Invalid {} format: {}", field_name, e)))
    }

    /// Validate timestamp format (YYYYMMDD-HH:MM:SS or YYYYMMDD-HH:MM:SS.sss)
    fn validate_timestamp(&self, timestamp: &[u8]) -> bool {
        let timestamp_str = match str::from_utf8(timestamp) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Basic length check
        if timestamp_str.len() != 17 && timestamp_str.len() != 21 {
            return false;
        }

        // Check date-time separator
        if !timestamp_str.is_char_boundary(8) || timestamp_str.as_bytes()[8] != b'-' {
            return false;
        }

        // Check time separators
        if !timestamp_str.is_char_boundary(11) || timestamp_str.as_bytes()[11] != b':' ||
           !timestamp_str.is_char_boundary(14) || timestamp_str.as_bytes()[14] != b':' {
            return false;
        }

        // If milliseconds are present, check decimal point
        if timestamp_str.len() == 21 && 
           (!timestamp_str.is_char_boundary(17) || timestamp_str.as_bytes()[17] != b'.') {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(msg_type: &str) -> Vec<u8> {
        // Create a valid FIX 4.2 message with SOH field separator
        format!(
            "8=FIX.4.2\x019=100\x0135={}\x0134=1\x0149=SENDER\x0156=TARGET\x0152=20240111-12:00:00\x0110=000\x01",
            msg_type
        ).into_bytes()
    }

    #[test]
    fn test_parse_valid_message() {
        let parser = FixParser::new();
        let message = create_test_message("A"); // Logon message
        let result = parser.parse(&message);
        assert!(result.is_ok());
        
        let validated = result.unwrap();
        assert_eq!(validated.msg_type, MessageType::Logon);
        assert_eq!(validated.sender_comp_id, "SENDER");
        assert_eq!(validated.target_comp_id, "TARGET");
        assert_eq!(validated.msg_seq_num, 1);
    }

    #[test]
    fn test_message_too_large() {
        let parser = FixParser::new();
        let large_message = vec![b'1'; 5000]; // Exceeds max size
        let result = parser.parse(&large_message);
        assert!(matches!(result, Err(FixError::MessageTooLarge)));
    }

    #[test]
    fn test_invalid_version() {
        let parser = FixParser::new();
        let mut message = create_test_message("A");
        // Modify FIX.4.2 to FIX.4.1
        message[2] = b'1';
        let result = parser.parse(&message);
        assert!(matches!(result, Err(FixError::InvalidVersion)));
    }

    #[test]
    fn test_invalid_sending_time() {
        let parser = FixParser::new();
        let mut message = create_test_message("A");
        // Corrupt the sending time field
        let time_start = message.windows(12).position(|w| w == b"52=").unwrap() + 3;
        message[time_start] = b'X';
        let result = parser.parse(&message);
        assert!(matches!(result, Err(FixError::InvalidFormat(_))));
    }

    #[test]
    fn test_missing_required_field() {
        let parser = FixParser::new();
        // Create message missing SenderCompID
        let message = b"8=FIX.4.2\x019=50\x0135=A\x0134=1\x0156=TARGET\x0152=20240111-12:00:00\x0110=000\x01";
        let result = parser.parse(message);
        assert!(matches!(result, Err(FixError::MissingField(_))));
    }

    #[test]
    fn test_validate_timestamp() {
        let parser = FixParser::new();
        
        // Valid timestamps
        assert!(parser.validate_timestamp(b"20240111-12:00:00"));
        assert!(parser.validate_timestamp(b"20240111-12:00:00.123"));

        // Invalid timestamps
        assert!(!parser.validate_timestamp(b"2024011112:00:00")); // Missing separator
        assert!(!parser.validate_timestamp(b"20240111-12:00")); // Missing seconds
        assert!(!parser.validate_timestamp(b"20240111-12:00:00.1234")); // Too many milliseconds
        assert!(!parser.validate_timestamp(b"2024011A-12:00:00")); // Invalid character
    }
}

    */