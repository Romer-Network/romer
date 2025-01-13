use super::types::*;
use fefix::prelude::*;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashSet;


/*  
/// The FixValidator performs business-level validation of FIX messages after they've been
/// parsed successfully. This includes checking message-specific required fields,
/// value ranges, and temporal validations.
pub struct FixValidator {
    // Set of authorized sender comp IDs that can send messages
    valid_senders: HashSet<String>,
    // Maximum allowed time difference between message SendingTime and current time
    max_time_diff: Duration,
}

impl FixValidator {
    pub fn new() -> Self {
        Self {
            valid_senders: HashSet::new(),
            max_time_diff: Duration::seconds(30),
        }
    }

    /// Register a new sender comp ID as valid
    pub fn register_sender(&mut self, sender_comp_id: String) {
        self.valid_senders.insert(sender_comp_id);
    }

    /// Validate a parsed message, performing message-type specific validation
    pub fn validate(&self, message: &ValidatedMessage) -> FixResult<()> {
        // First check if the sender is authorized
        if !self.valid_senders.contains(&message.sender_comp_id) {
            return Err(FixError::InvalidFormat(
                format!("Unauthorized SenderCompID: {}", message.sender_comp_id)
            ));
        }

        // Perform message-type specific validation
        match message.msg_type {
            MessageType::Logon => self.validate_logon(&message.message),
            MessageType::NewOrderSingle => self.validate_new_order(&message.message),
            MessageType::MarketDataRequest => self.validate_market_data(&message.message),
            MessageType::OrderCancelRequest => self.validate_cancel_order(&message.message),
            // Session messages generally don't need extensive validation
            MessageType::Heartbeat | 
            MessageType::TestRequest |
            MessageType::ResendRequest |
            MessageType::SequenceReset |
            MessageType::Logout => self.validate_sending_time(&message.message),
        }
    }

    /// Validate logon message - checks heartbeat interval and encryption
    fn validate_logon(&self, message: &fefix::tagvalue::Message) -> FixResult<()> {
        // Validate required heartbeat interval
        let heartbeat = message.get_field::<HeartBtInt>()
            .map_err(|_| FixError::MissingField("HeartBtInt".to_string()))?
            .as_str()
            .parse::<u32>()
            .map_err(|_| FixError::InvalidFormat("Invalid HeartBtInt".to_string()))?;

        // Heartbeat must be between 10 and 60 seconds
        if heartbeat < 10 || heartbeat > 60 {
            return Err(FixError::InvalidFormat(
                "HeartBtInt must be between 10 and 60 seconds".to_string()
            ));
        }

        // Validate sending time is recent
        self.validate_sending_time(message)
    }

    /// Validate new order single message - checks required order fields
    fn validate_new_order(&self, message: &fefix::tagvalue::Message) -> FixResult<()> {
        // Check all required fields are present and valid
        let symbol = message.get_field::<Symbol>()
            .map_err(|_| FixError::MissingField("Symbol".to_string()))?;

        let side = message.get_field::<Side>()
            .map_err(|_| FixError::MissingField("Side".to_string()))?;

        let order_qty = message.get_field::<OrderQty>()
            .map_err(|_| FixError::MissingField("OrderQty".to_string()))?
            .as_str()
            .parse::<f64>()
            .map_err(|_| FixError::InvalidFormat("Invalid OrderQty".to_string()))?;

        let ord_type = message.get_field::<OrdType>()
            .map_err(|_| FixError::MissingField("OrdType".to_string()))?;

        // Validate order quantity is positive
        if order_qty <= 0.0 {
            return Err(FixError::InvalidFormat("OrderQty must be positive".to_string()));
        }

        // If it's a limit order, price is required
        if ord_type.as_str() == "2" {  // 2 = Limit
            let _ = message.get_field::<Price>()
                .map_err(|_| FixError::MissingField("Price required for limit orders".to_string()))?;
        }

        self.validate_sending_time(message)
    }

    /// Validate market data request message
    fn validate_market_data(&self, message: &fefix::tagvalue::Message) -> FixResult<()> {
        // Validate required fields
        let _ = message.get_field::<MDReqID>()
            .map_err(|_| FixError::MissingField("MDReqID".to_string()))?;

        let subscription_type = message.get_field::<SubscriptionRequestType>()
            .map_err(|_| FixError::MissingField("SubscriptionRequestType".to_string()))?
            .as_str()
            .parse::<char>()
            .map_err(|_| FixError::InvalidFormat("Invalid SubscriptionRequestType".to_string()))?;

        // Validate subscription type is valid (0 = Snapshot, 1 = Subscribe, 2 = Unsubscribe)
        if !['0', '1', '2'].contains(&subscription_type) {
            return Err(FixError::InvalidFormat("Invalid SubscriptionRequestType".to_string()));
        }

        let market_depth = message.get_field::<MarketDepth>()
            .map_err(|_| FixError::MissingField("MarketDepth".to_string()))?
            .as_str()
            .parse::<u32>()
            .map_err(|_| FixError::InvalidFormat("Invalid MarketDepth".to_string()))?;

        // Validate market depth is reasonable (1-50 levels)
        if market_depth < 1 || market_depth > 50 {
            return Err(FixError::InvalidFormat("MarketDepth must be between 1 and 50".to_string()));
        }

        self.validate_sending_time(message)
    }

    /// Validate cancel order request message
    fn validate_cancel_order(&self, message: &fefix::tagvalue::Message) -> FixResult<()> {
        // Original order ID or client order ID must be present
        if message.get_field::<OrderID>().is_err() && message.get_field::<OrigClOrdID>().is_err() {
            return Err(FixError::MissingField(
                "Either OrderID or OrigClOrdID must be present".to_string()
            ));
        }

        // Validate required fields
        let _ = message.get_field::<Symbol>()
            .map_err(|_| FixError::MissingField("Symbol".to_string()))?;

        let _ = message.get_field::<Side>()
            .map_err(|_| FixError::MissingField("Side".to_string()))?;

        self.validate_sending_time(message)
    }

    /// Validate message sending time is within acceptable range
    fn validate_sending_time(&self, message: &fefix::tagvalue::Message) -> FixResult<()> {
        let sending_time = message.get_field::<SendingTime>()
            .map_err(|_| FixError::MissingField("SendingTime".to_string()))?;

        // Parse the UTC timestamp from the message
        let sending_time = DateTime::parse_from_str(
            sending_time.as_str(),
            "%Y%m%d-%H:%M:%S%.3f"
        ).map_err(|_| FixError::InvalidFormat("Invalid SendingTime format".to_string()))?;

        // Convert to UTC for comparison
        let sending_time_utc = sending_time.with_timezone(&Utc);
        let current_time = Utc::now();

        // Check if sending time is too far in past or future
        if (sending_time_utc - current_time).abs() > self.max_time_diff {
            return Err(FixError::InvalidFormat("SendingTime too far from current time".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fefix::tagvalue::{Config, Dictionary};

    // Helper function to create a basic FIX message for testing
    fn create_test_message(msg_type: MessageType) -> ValidatedMessage {
        ValidatedMessage {
            msg_type,
            message: fefix::tagvalue::Message::new(Dictionary::fix42()),
            sender_comp_id: "TESTCOMPID".to_string(),
            target_comp_id: "ROMER".to_string(),
            msg_seq_num: 1,
        }
    }

    #[test]
    fn test_unauthorized_sender() {
        let validator = FixValidator::new();
        let message = create_test_message(MessageType::Logon);
        
        assert!(matches!(
            validator.validate(&message),
            Err(FixError::InvalidFormat(msg)) if msg.contains("Unauthorized SenderCompID")
        ));
    }

    // Add more tests as needed for specific message validation...
}

    */