use romer_common::types::fix::{FixConfig, MessageType, ValidatedMessage, utils};
use rand::Rng;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

/// FixMockGenerator provides utilities for creating mock FIX messages for testing
/// and development purposes. All messages are created with valid structure,
/// proper checksums, and realistic data to simulate production scenarios.
pub struct FixMockGenerator;

impl FixMockGenerator {
    /// Creates a mock Logon message (35=A) used to initiate a FIX session.
    /// The Logon message includes essential session parameters like heartbeat
    /// interval and encryption method, along with the standard header fields.
    /// 
    /// # Arguments
    /// * `config` - The FIX configuration containing sender/target information
    pub fn mock_logon(config: &FixConfig) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();
        
        // Construct the message body with all required Logon fields:
        // 8=FIX Version        - Begin string
        // 9=Length            - Body length (calculated later)
        // 35=A               - Message type (Logon)
        // 49=SenderCompID    - Sender ID
        // 56=TargetCompID    - Target ID
        // 34=SeqNum          - Message sequence number
        // 52=Time            - Sending time
        // 108=30            - Heartbeat interval (30 seconds)
        // 98=0              - Encryption method (none)
        let msg = format!(
            "8=FIX.{}|9=0|35=A|49={}|56={}|34={}|52={}|108=30|98=0|",
            config.fix_version,
            config.sender_comp_id,
            config.target_comp_id,
            msg_seq_num,
            timestamp
        );

        // Calculate and append the message checksum (tag 10)
        let raw_data = format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes()))
            .into_bytes();

        ValidatedMessage {
            msg_type: MessageType::Logon,
            sender_comp_id: config.sender_comp_id.clone(),
            target_comp_id: config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }

    /// Creates a mock Logout message (35=5) used to terminate a FIX session.
    /// Includes an optional text field explaining the logout reason.
    pub fn mock_logout(config: &FixConfig) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();
        
        let msg = format!(
            "8=FIX.{}|9=0|35=5|49={}|56={}|34={}|52={}|58=Normal Logout|",
            config.fix_version,
            config.sender_comp_id,
            config.target_comp_id,
            msg_seq_num,
            timestamp
        );

        let raw_data = format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes()))
            .into_bytes();

        ValidatedMessage {
            msg_type: MessageType::Logout,
            sender_comp_id: config.sender_comp_id.clone(),
            target_comp_id: config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }

    /// Creates a mock New Order Single message (35=D) representing a new trade order.
    /// Generates realistic order details including symbol, price, and quantity.
    pub fn mock_new_order_single(config: &FixConfig) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();
        let client_order_id = format!("ORDER{}", Uuid::new_v4().simple());
        let price: f64 = (rng.gen_range(10.0..100.0) * 100.0) / 100.0;
        let quantity = rng.gen_range(100..10_000);

        let msg = format!(
            "8=FIX.{}|9=0|35=D|49={}|56={}|34={}|52={}|11={}|55=AAPL|54=1|38={}|40=2|44={}|59=0|",
            config.fix_version,
            config.sender_comp_id,
            config.target_comp_id,
            msg_seq_num,
            timestamp,
            client_order_id,
            quantity,
            price
        );

        let raw_data = format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes()))
            .into_bytes();

        ValidatedMessage {
            msg_type: MessageType::NewOrderSingle,
            sender_comp_id: config.sender_comp_id.clone(),
            target_comp_id: config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }

    /// Creates a mock Market Data Request message (35=V) used to subscribe
    /// to market data for specified symbols.
    pub fn mock_market_data_request(config: &FixConfig) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();
        let request_id = format!("REQ{}", Uuid::new_v4().simple());

        let msg = format!(
            "8=FIX.{}|9=0|35=V|49={}|56={}|34={}|52={}|262={}|263=1|264=0|267=2|269=0|269=1|146=2|55=AAPL|55=GOOGL|",
            config.fix_version,
            config.sender_comp_id,
            config.target_comp_id,
            msg_seq_num,
            timestamp,
            request_id
        );

        let raw_data = format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes()))
            .into_bytes();

        ValidatedMessage {
            msg_type: MessageType::MarketDataRequest,
            sender_comp_id: config.sender_comp_id.clone(),
            target_comp_id: config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }

    /// Creates a mock Heartbeat message (35=0) used to maintain session activity
    /// during periods of low message traffic.
    pub fn mock_heartbeat(config: &FixConfig) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();

        let msg = format!(
            "8=FIX.{}|9=0|35=0|49={}|56={}|34={}|52={}|",
            config.fix_version,
            config.sender_comp_id,
            config.target_comp_id,
            msg_seq_num,
            timestamp
        );

        let raw_data = format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes()))
            .into_bytes();

        ValidatedMessage {
            msg_type: MessageType::Heartbeat,
            sender_comp_id: config.sender_comp_id.clone(),
            target_comp_id: config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a standard configuration for testing
    fn create_test_config() -> FixConfig {
        FixConfig {
            fix_version: "4.2".to_string(),
            sender_comp_id: "SENDER".to_string(),
            target_comp_id: "TARGET".to_string(),
        }
    }

    #[test]
    fn test_mock_logon() {
        let config = create_test_config();
        let message = FixMockGenerator::mock_logon(&config);
        
        assert_eq!(message.msg_type, MessageType::Logon);
        
        let fields = utils::parse_message_fields(&message.raw_data);
        assert_eq!(fields.get(&35).unwrap(), "A"); // MsgType
        assert_eq!(fields.get(&49).unwrap(), &config.sender_comp_id);
        assert_eq!(fields.get(&56).unwrap(), &config.target_comp_id);
        assert!(fields.contains_key(&108)); // HeartBtInt
        assert!(fields.contains_key(&98));  // EncryptMethod
    }

    #[test]
    fn test_mock_new_order_single() {
        let config = create_test_config();
        let message = FixMockGenerator::mock_new_order_single(&config);
        
        assert_eq!(message.msg_type, MessageType::NewOrderSingle);
        
        let fields = utils::parse_message_fields(&message.raw_data);
        assert_eq!(fields.get(&35).unwrap(), "D");
        assert!(fields.contains_key(&11)); // ClOrdID
        assert!(fields.contains_key(&44)); // Price
        assert!(fields.contains_key(&38)); // OrderQty
    }
}