use fefix::{
    Dictionary,
    tagvalue::{
        Config, EncoderHandle, Field, FixValue, Message, MessageBuilder, RepeatingGroup, Value,
    },
};
use std::net::TcpListener;
use std::io::{Read, Write};
use tokio::net::TcpStream;
use tracing::{info, warn, error};

// FIX message handling configuration
const FIX_VERSION: &str = "FIX.4.4";
const SENDER_COMP_ID: &str = "ROMER";
const TARGET_COMP_ID: &str = "CLIENT";

// Initialize tracing for logging
fn init_logging() {
    tracing_subscriber::fmt::init();
}

// Create a basic FIX message configuration
fn create_fix_config() -> Config {
    Config::new()
        .with_sender_comp_id(SENDER_COMP_ID)
        .with_target_comp_id(TARGET_COMP_ID)
        .with_version(FIX_VERSION)
}

// Handle a FIX Logon message
fn handle_logon(msg: &Message) -> Option<Message> {
    info!("Received Logon message");
    
    // Create Logon response
    let mut builder = MessageBuilder::new("A", create_fix_config());  // "A" is message type for Logon
    builder.with_field(98, Value::Int(0));  // EncryptMethod: None
    builder.with_field(108, Value::Int(30)); // HeartBtInt: 30 seconds
    
    Some(builder.build())
}

// Handle a FIX Heartbeat message
fn handle_heartbeat(msg: &Message) -> Option<Message> {
    info!("Received Heartbeat");
    None  // No response needed for heartbeat
}

// Main message handler
fn handle_fix_message(msg: Message) -> Option<Message> {
    match msg.msg_type() {
        "A" => handle_logon(&msg),
        "0" => handle_heartbeat(&msg),
        unknown => {
            warn!("Received unknown message type: {}", unknown);
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    info!("Starting Rømer Chain FIX sequencer");

    // Create TCP listener
    let listener = TcpListener::bind("127.0.0.1:9898")?;
    info!("Listening for FIX connections on port 9898");

    // Accept connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                info!("New connection established");
                
                // Handle connection in new task
                tokio::spawn(async move {
                    let mut buffer = [0; 4096];
                    
                    loop {
                        match stream.read(&mut buffer) {
                            Ok(n) if n == 0 => {
                                info!("Connection closed by client");
                                break;
                            }
                            Ok(n) => {
                                // Parse FIX message
                                if let Ok(msg) = Message::from_bytes(&buffer[..n]) {
                                    // Handle message and get optional response
                                    if let Some(response) = handle_fix_message(msg) {
                                        // Send response
                                        if let Err(e) = stream.write_all(&response.to_bytes()) {
                                            error!("Error sending response: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error reading from socket: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}