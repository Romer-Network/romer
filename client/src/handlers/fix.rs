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
        println!("\nGenerated FIX Message:");
        println!("Message Type: {:?}", message.msg_type);
        println!("Sequence Number: {}", message.msg_seq_num);
        println!("Raw Message:");
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

    fn display_message(&self, message: &ValidatedMessage) {
        println!("\nGenerated FIX Message:");
        println!("Message Type: {:?}", message.msg_type);
        println!("Sequence Number: {}", message.msg_seq_num);
        println!("Raw Message:");
        println!("{}", String::from_utf8_lossy(&message.raw_data));
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

    fn display_message(&self, message: &ValidatedMessage) {
        println!("\nGenerated FIX Message:");
        println!("Message Type: {:?}", message.msg_type);
        println!("Sequence Number: {}", message.msg_seq_num);
        println!("Raw Message:");
        println!("{}", String::from_utf8_lossy(&message.raw_data));
    }
}

impl Handler for HeartbeatHandler {
    fn handle(&self) -> io::Result<()> {
        let heartbeat = self.mock_generator.create_heartbeat();
        self.display_message(&heartbeat);
        Ok(())
    }
}
