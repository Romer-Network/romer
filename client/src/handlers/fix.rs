use crate::handlers::Handler;
use rand::Rng;
use romer_common::types::fix::{utils, FixConfig, MessageType, ValidatedMessage};
use std::io::{self, Write};
use uuid::Uuid;

// The FIX message generator handles creating properly formatted FIX messages
pub struct MockGenerator {
    config: FixConfig,
}

impl MockGenerator {
    pub fn new(config: FixConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self {
            config: FixConfig {
                fix_version: "4.2".to_string(),
                sender_comp_id: "ROMER".to_string(),
                target_comp_id: "MARKET".to_string(),
            },
        }
    }

    // Creates a FIX logon message with authentication information
    pub fn create_logon(&self) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();

        // Construct the logon message following FIX 4.2 format
        let msg = format!(
            "8=FIX.{}|9=0|35=A|49={}|56={}|34={}|52={}|108=30|98=0|",
            self.config.fix_version,
            self.config.sender_comp_id,
            self.config.target_comp_id,
            msg_seq_num,
            timestamp
        );

        let raw_data =
            format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes())).into_bytes();

        ValidatedMessage {
            msg_type: MessageType::Logon,
            sender_comp_id: self.config.sender_comp_id.clone(),
            target_comp_id: self.config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }

    // Creates a FIX logout message for terminating sessions
    pub fn create_logout(&self) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();

        let msg = format!(
            "8=FIX.{}|9=0|35=5|49={}|56={}|34={}|52={}|58=Normal Logout|",
            self.config.fix_version,
            self.config.sender_comp_id,
            self.config.target_comp_id,
            msg_seq_num,
            timestamp
        );

        let raw_data =
            format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes())).into_bytes();

        ValidatedMessage {
            msg_type: MessageType::Logout,
            sender_comp_id: self.config.sender_comp_id.clone(),
            target_comp_id: self.config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }

    // Creates a FIX heartbeat message for maintaining session connectivity
    pub fn create_heartbeat(&self) -> ValidatedMessage {
        let mut rng = rand::thread_rng();
        let msg_seq_num = rng.gen_range(1..100_000);
        let timestamp = utils::generate_timestamp();

        let msg = format!(
            "8=FIX.{}|9=0|35=0|49={}|56={}|34={}|52={}|",
            self.config.fix_version,
            self.config.sender_comp_id,
            self.config.target_comp_id,
            msg_seq_num,
            timestamp
        );

        let raw_data =
            format!("{}10={}|", msg, utils::calculate_checksum(msg.as_bytes())).into_bytes();

        ValidatedMessage {
            msg_type: MessageType::Heartbeat,
            sender_comp_id: self.config.sender_comp_id.clone(),
            target_comp_id: self.config.target_comp_id.clone(),
            msg_seq_num,
            raw_data,
        }
    }
}

// Handles FIX session logon operations
pub struct LogonHandler {
    mock_generator: MockGenerator,
}

impl LogonHandler {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            mock_generator: MockGenerator::default(),
        })
    }

    // Gets FIX session configuration from user input
    fn get_session_config(&self) -> io::Result<FixConfig> {
        println!("\nEnter FIX session details (or press Enter for defaults):");

        print!("SenderCompID [ROMER]: ");
        io::stdout().flush()?;
        let mut sender = String::new();
        io::stdin().read_line(&mut sender)?;
        let sender = sender.trim();

        print!("TargetCompID [MARKET]: ");
        io::stdout().flush()?;
        let mut target = String::new();
        io::stdin().read_line(&mut target)?;
        let target = target.trim();

        Ok(FixConfig {
            fix_version: "4.2".to_string(),
            sender_comp_id: if sender.is_empty() {
                "ROMER".to_string()
            } else {
                sender.to_string()
            },
            target_comp_id: if target.is_empty() {
                "MARKET".to_string()
            } else {
                target.to_string()
            },
        })
    }

    // Displays a formatted FIX message
    fn display_message(&self, message: &ValidatedMessage) -> io::Result<()> {
        println!("\nGenerated FIX Logon Message Details:");
        println!(
            "\nMessage Type: {:?} (35=A - Used to initiate a FIX session)",
            message.msg_type
        );
        println!("\nHeader Fields:");

        // Parse the raw message into fields
        let fields = utils::parse_message_fields(&message.raw_data);

        // Display and explain each field
        if let Some(begin_string) = fields.get(&8) {
            println!("  BeginString (8): {} - FIX protocol version", begin_string);
        }

        if let Some(body_length) = fields.get(&9) {
            println!("  BodyLength (9): {} - Length of message body", body_length);
        }

        if let Some(sender_comp_id) = fields.get(&49) {
            println!(
                "  SenderCompID (49): {} - Unique identifier for the sending firm",
                sender_comp_id
            );
        }

        if let Some(target_comp_id) = fields.get(&56) {
            println!(
                "  TargetCompID (56): {} - Unique identifier for the target firm",
                target_comp_id
            );
        }

        if let Some(msg_seq_num) = fields.get(&34) {
            println!(
                "  MsgSeqNum (34): {} - Message sequence number",
                msg_seq_num
            );
        }

        if let Some(sending_time) = fields.get(&52) {
            println!(
                "  SendingTime (52): {} - Time message was sent",
                sending_time
            );
        }

        println!("\nLogon-Specific Fields:");
        if let Some(heartbeat_int) = fields.get(&108) {
            println!(
                "  HeartBtInt (108): {} - Heartbeat interval in seconds",
                heartbeat_int
            );
        }

        if let Some(encrypt_method) = fields.get(&98) {
            let encrypt_desc = match encrypt_method.as_str() {
                "0" => "None/Other",
                "1" => "PKCS",
                "2" => "DES",
                "3" => "PKCS/DES",
                "4" => "PGP/DES",
                "5" => "PGP/DES-MD5",
                _ => "Unknown",
            };
            println!(
                "  EncryptMethod (98): {} - {} - Method of encryption",
                encrypt_method, encrypt_desc
            );
        }

        println!("\nTrailer Fields:");
        if let Some(checksum) = fields.get(&10) {
            println!(
                "  CheckSum (10): {} - Message checksum for validation",
                checksum
            );
        }

        println!("\nRaw Message (for reference):");
        println!("{}", String::from_utf8_lossy(&message.raw_data));

        Ok(())
    }
}

impl Handler for LogonHandler {
    fn handle(&self) -> io::Result<()> {
        let config = self.get_session_config()?;
        let generator = MockGenerator::new(config);
        let logon = generator.create_logon();
        self.display_message(&logon);
        Ok(())
    }
}

// Handles FIX session logout operations
pub struct LogoutHandler {
    mock_generator: MockGenerator,
}

impl LogoutHandler {
    pub fn new() -> Self {
        Self {
            mock_generator: MockGenerator::default(),
        }
    }

    fn display_message(&self, message: &ValidatedMessage) -> io::Result<()> {
        println!("\nGenerated FIX Logout Message Details:");
        println!(
            "\nMessage Type: {:?} (35=5 - Used to terminate a FIX session)",
            message.msg_type
        );
        println!("\nHeader Fields:");

        // Parse the raw message into fields
        let fields = utils::parse_message_fields(&message.raw_data);

        // Display and explain each field
        if let Some(begin_string) = fields.get(&8) {
            println!("  BeginString (8): {} - FIX protocol version", begin_string);
        }

        if let Some(body_length) = fields.get(&9) {
            println!("  BodyLength (9): {} - Length of message body", body_length);
        }

        if let Some(sender_comp_id) = fields.get(&49) {
            println!(
                "  SenderCompID (49): {} - Unique identifier for the sending firm",
                sender_comp_id
            );
        }

        if let Some(target_comp_id) = fields.get(&56) {
            println!(
                "  TargetCompID (56): {} - Unique identifier for the target firm",
                target_comp_id
            );
        }

        if let Some(msg_seq_num) = fields.get(&34) {
            println!(
                "  MsgSeqNum (34): {} - Message sequence number",
                msg_seq_num
            );
        }

        if let Some(sending_time) = fields.get(&52) {
            println!(
                "  SendingTime (52): {} - Time message was sent",
                sending_time
            );
        }

        println!("\nLogout-Specific Fields:");
        if let Some(text) = fields.get(&58) {
            println!(
                "  Text (58): {} - Free format text explaining the logout",
                text
            );
        }

        println!("\nTrailer Fields:");
        if let Some(checksum) = fields.get(&10) {
            println!(
                "  CheckSum (10): {} - Message checksum for validation",
                checksum
            );
        }

        println!("\nRaw Message (for reference):");
        println!("{}", String::from_utf8_lossy(&message.raw_data));

        Ok(())
    }
}

impl Handler for LogoutHandler {
    fn handle(&self) -> io::Result<()> {
        let logout = self.mock_generator.create_logout();
        self.display_message(&logout);
        Ok(())
    }
}

// Handles FIX heartbeat operations
pub struct HeartbeatHandler {
    mock_generator: MockGenerator,
}

impl HeartbeatHandler {
    pub fn new() -> Self {
        Self {
            mock_generator: MockGenerator::default(),
        }
    }

    // Display function to show detailed heartbeat message information
    fn display_message(&self, message: &ValidatedMessage) -> io::Result<()> {
        // Print the message type heading with explanation
        println!("\nGenerated FIX Heartbeat Message Details:");
        println!(
            "\nMessage Type: {:?} (35=0 - Used to maintain FIX session activity)",
            message.msg_type
        );

        // Parse the raw FIX message into a field map for easier access
        let fields = utils::parse_message_fields(&message.raw_data);

        // Display standard header fields with explanations
        println!("\nHeader Fields:");

        // BeginString (tag 8) - FIX protocol version
        if let Some(begin_string) = fields.get(&8) {
            println!("  BeginString (8): {} - FIX protocol version", begin_string);
        }

        // BodyLength (tag 9) - Message body length
        if let Some(body_length) = fields.get(&9) {
            println!("  BodyLength (9): {} - Length of message body", body_length);
        }

        // SenderCompID (tag 49) - Message sender identification
        if let Some(sender_comp_id) = fields.get(&49) {
            println!(
                "  SenderCompID (49): {} - Unique identifier for the sending firm",
                sender_comp_id
            );
        }

        // TargetCompID (tag 56) - Message recipient identification
        if let Some(target_comp_id) = fields.get(&56) {
            println!(
                "  TargetCompID (56): {} - Unique identifier for the target firm",
                target_comp_id
            );
        }

        // MsgSeqNum (tag 34) - Message sequence number for gap detection
        if let Some(msg_seq_num) = fields.get(&34) {
            println!(
                "  MsgSeqNum (34): {} - Message sequence number",
                msg_seq_num
            );
        }

        // SendingTime (tag 52) - Time of message transmission
        if let Some(sending_time) = fields.get(&52) {
            println!(
                "  SendingTime (52): {} - Time message was sent",
                sending_time
            );
        }

        // Heartbeat messages are unique in that they typically have no message-specific fields
        println!("\nMessage-Specific Fields:");
        println!("  [None] - Heartbeat messages do not contain any additional fields");
        println!("          They serve only to maintain session activity");

        // Display trailer fields
        println!("\nTrailer Fields:");

        // CheckSum (tag 10) - Message integrity verification
        if let Some(checksum) = fields.get(&10) {
            println!(
                "  CheckSum (10): {} - Message checksum for validation",
                checksum
            );
        }

        // Show the raw message for reference purposes
        println!("\nRaw Message (for reference):");
        println!("{}", String::from_utf8_lossy(&message.raw_data));

        Ok(())
    }
}

impl Handler for HeartbeatHandler {
    fn handle(&self) -> io::Result<()> {
        let heartbeat = self.mock_generator.create_heartbeat();
        self.display_message(&heartbeat);
        Ok(())
    }
}
