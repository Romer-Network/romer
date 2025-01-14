use crate::handlers::Handler;
use rand::Rng;
use romer_common::{error::RomerResult, fix::mock::FixMockGenerator, types::fix::{utils, FixConfig, MessageType, ValidatedMessage}};
use std::{
    io::{self, Write}
};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use uuid::Uuid;
use romer_common::{
    types::org::{Organization, OrganizationType},
    storage::journal::RomerJournal,
};


// Handles FIX session logon operations
pub struct LogonHandler {
    mock_generator: FixMockGenerator,
}

impl LogonHandler {
    pub fn new() -> io::Result<Self> {

        let config = FixConfig::default();
        let mock_generator = FixMockGenerator::new(config);
        Ok(Self {
            mock_generator,
        })
    }

    // New method to send message and get response
    async fn send_message(&self, message: &ValidatedMessage) -> io::Result<String> {
        // Connect to the local sequencer
        let mut stream = TcpStream::connect("127.0.0.1:9878").await?;

        // Send the raw message
        stream.write_all(&message.raw_data).await?;

        // Read the response
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer).await?;

        // Convert response to string
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
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
    fn handle(&mut self) -> RomerResult<()> {
        // Get config and create message like before
        let config = self.get_session_config()?;
        let generator = FixMockGenerator::new(config);
        let logon = generator.mock_logon();

        // Display the message we're about to send
        self.display_message(&logon)?;

        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;

        // Send message and display response
        println!("\nSending message to sequencer...");
        match runtime.block_on(self.send_message(&logon)) {
            Ok(response) => {
                println!("\nReceived response from sequencer:");
                println!("{}", response);
            }
            Err(e) => println!("Error communicating with sequencer: {}", e),
        }

        Ok(())
    }
}

// Handles FIX session logout operations
pub struct LogoutHandler {
    mock_generator: FixMockGenerator,
}

impl LogoutHandler {
    pub fn new() -> Self {
        let config = FixConfig::default();
        let mock_generator = FixMockGenerator::new(config);
        Self {
            mock_generator,
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
    fn handle(&mut self) -> RomerResult<()> {
        
        let logout = self.mock_generator.mock_logout();
        self.display_message(&logout);
        Ok(())
    }
}

// Handles FIX heartbeat operations
pub struct HeartbeatHandler {
    mock_generator: FixMockGenerator,
}

impl HeartbeatHandler {
    pub fn new() -> Self {
        let config = FixConfig::default();
        let mock_generator = FixMockGenerator::new(config);
        Self {
            mock_generator,
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
    fn handle(&mut self) -> RomerResult<()> {
        let heartbeat = self.mock_generator.mock_heartbeat();
        self.display_message(&heartbeat);
        Ok(())
    }
}