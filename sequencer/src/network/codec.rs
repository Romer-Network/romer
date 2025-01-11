// src/network/codec.rs

use bytes::{BytesMut, Buf, BufMut};
use std::str;
use crate::network::types::{NetworkError, NetworkResult};
use tracing::{debug, warn};

/// Maximum length for a single FIX message
const MAX_MESSAGE_LENGTH: usize = 4096;

/// Special characters used in FIX protocol
const SOH: u8 = 0x01;  // Start of header (field separator)
const EQUALS: u8 = b'=';  // Key-value separator

/// Handles FIX protocol message encoding and decoding
pub struct FixCodec {
    /// Maximum message size we'll accept
    max_message_size: usize,
    /// Current state of message parsing
    parse_state: ParseState,
}

/// Tracks the state of message parsing
#[derive(Debug, Clone)]
enum ParseState {
    /// Looking for start of message
    WaitingForBegin,
    /// Reading message body length
    ReadingLength {
        /// Position of body length field start
        start_pos: usize,
    },
    /// Reading message body
    ReadingBody {
        /// Expected length of message body
        body_length: usize,
        /// Position where body starts
        start_pos: usize,
    },
}

impl FixCodec {
    /// Create a new FIX codec
    pub fn new() -> Self {
        Self {
            max_message_size: MAX_MESSAGE_LENGTH,
            parse_state: ParseState::WaitingForBegin,
        }
    }

    /// Attempt to extract the next complete message from a buffer
    pub fn try_parse(buf: &mut BytesMut) -> NetworkResult<Option<BytesMut>> {
        // We need at least "8=FIX" to start
        if buf.len() < 5 {
            return Ok(None);
        }

        // Find the start of a FIX message
        let mut pos = 0;
        while pos + 5 <= buf.len() {
            if &buf[pos..pos+2] == b"8=" && buf[pos+4] == SOH {
                // Found potential start, validate FIX version
                if let Ok(version) = str::from_utf8(&buf[pos+2..pos+4]) {
                    if version.starts_with("FIX") {
                        break;
                    }
                }
            }
            pos += 1;
        }

        // If we didn't find a start marker, keep waiting
        if pos + 5 > buf.len() {
            return Ok(None);
        }

        // Look for body length field (tag 9)
        let mut length_start = None;
        let mut length_end = None;
        let mut i = pos + 5;
        
        while i + 3 <= buf.len() {
            if &buf[i..i+2] == b"9=" {
                length_start = Some(i + 2);
                // Find the SOH that ends the length field
                while i < buf.len() {
                    if buf[i] == SOH {
                        length_end = Some(i);
                        break;
                    }
                    i += 1;
                }
                break;
            }
            i += 1;
        }

        // If we don't have a complete length field yet, keep waiting
        let (length_start, length_end) = match (length_start, length_end) {
            (Some(start), Some(end)) => (start, end),
            _ => return Ok(None),
        };

        // Parse the body length
        let body_length = match str::from_utf8(&buf[length_start..length_end]) {
            Ok(len_str) => match len_str.parse::<usize>() {
                Ok(len) => len,
                Err(_) => {
                    warn!("Invalid body length format");
                    return Err(NetworkError::InvalidFormat("Invalid body length".into()));
                }
            },
            Err(_) => {
                warn!("Invalid UTF-8 in body length");
                return Err(NetworkError::InvalidFormat("Invalid body length encoding".into()));
            }
        };

        // Validate message size
        if body_length > MAX_MESSAGE_LENGTH {
            warn!(length = body_length, "Message exceeds maximum size");
            return Err(NetworkError::MessageTooLarge { size: body_length });
        }

        // Calculate where message should end
        let msg_end = length_end + body_length + 1;
        if buf.len() < msg_end {
            // Don't have complete message yet
            return Ok(None);
        }

        // Verify checksum field exists and is valid
        if !Self::verify_checksum(&buf[pos..msg_end]) {
            warn!("Invalid message checksum");
            return Err(NetworkError::InvalidFormat("Invalid checksum".into()));
        }

        // Extract the complete message
        let message = buf.split_to(msg_end);
        debug!(length = message.len(), "Extracted complete FIX message");

        Ok(Some(message))
    }

    /// Calculate and verify message checksum
    fn verify_checksum(data: &[u8]) -> bool {
        // Find the checksum field
        let mut i = data.len() - 7;  // Minimum checksum field length
        while i > 0 {
            if &data[i..i+3] == b"10=" {
                // Parse the expected checksum
                if let Ok(expected) = str::from_utf8(&data[i+3..i+6])
                    .map(|s| u8::from_str_radix(s, 16))
                {
                    match expected {
                        Ok(expected) => {
                            // Calculate actual checksum (sum of all bytes modulo 256)
                            let actual: u8 = data[..i]
                                .iter()
                                .fold(0u8, |sum, &byte| sum.wrapping_add(byte));
                            
                            return expected == actual;
                        }
                        Err(_) => return false,
                    }
                }
                return false;
            }
            i -= 1;
        }
        false
    }

    /// Format an outgoing FIX message
    pub fn format_message(msg: &[u8]) -> NetworkResult<BytesMut> {
        // Validate basic message format
        if !msg.starts_with(b"8=FIX") {
            return Err(NetworkError::InvalidFormat("Missing FIX version".into()));
        }

        // Calculate and append checksum if needed
        let mut buf = BytesMut::with_capacity(msg.len() + 7);
        buf.put_slice(msg);
        
        if !msg.ends_with(SOH) {
            buf.put_u8(SOH);
        }

        // Only add checksum if it's not already present
        if !Self::has_checksum(&buf) {
            let sum: u8 = buf.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
            buf.put_slice(b"10=");
            buf.put_slice(format!("{:03X}", sum).as_bytes());
            buf.put_u8(SOH);
        }

        Ok(buf)
    }

    /// Check if message already has a checksum field
    fn has_checksum(data: &[u8]) -> bool {
        let mut i = data.len() - 7;  // Minimum checksum field length
        while i > 0 {
            if &data[i..i+3] == b"10=" {
                return true;
            }
            i -= 1;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_extraction() {
        let mut buf = BytesMut::from(&b"8=FIX.4.2\x019=5\x0135=0\x0110=31\x01"[..]);
        let result = FixCodec::try_parse(&mut buf).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_partial_message() {
        let mut buf = BytesMut::from(&b"8=FIX.4.2\x019=5\x0135=0"[..]);
        let result = FixCodec::try_parse(&mut buf).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_message_formatting() {
        let msg = b"8=FIX.4.2\x019=5\x0135=0\x01";
        let result = FixCodec::format_message(msg).unwrap();
        assert!(result.ends_with(SOH));
        assert!(FixCodec::verify_checksum(&result));
    }

    #[test]
    fn test_invalid_message() {
        let mut buf = BytesMut::from(&b"invalid message"[..]);
        let result = FixCodec::try_parse(&mut buf);
        assert!(result.is_ok());  // Should return None, not error
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_checksum_verification() {
        let msg = b"8=FIX.4.2\x019=5\x0135=0\x0110=31\x01";
        assert!(FixCodec::verify_checksum(msg));
    }

    #[test]
    fn test_multiple_messages() {
        let mut buf = BytesMut::from(
            &b"8=FIX.4.2\x019=5\x0135=0\x0110=31\x018=FIX.4.2\x019=5\x0135=0\x0110=31\x01"[..]
        );
        
        // First message
        let msg1 = FixCodec::try_parse(&mut buf).unwrap();
        assert!(msg1.is_some());
        
        // Second message
        let msg2 = FixCodec::try_parse(&mut buf).unwrap();
        assert!(msg2.is_some());
        
        // No more messages
        let msg3 = FixCodec::try_parse(&mut buf).unwrap();
        assert!(msg3.is_none());
    }
}