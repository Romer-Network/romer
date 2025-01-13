use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};
use romer_common::types::fix::MessageType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your existing setup code remains the same
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    let host = std::env::var("SEQUENCER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SEQUENCER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9878);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                info!("Accepted connection from: {}", addr);
                
                // Enable TCP_NODELAY for better latency
                if let Err(e) = socket.set_nodelay(true) {
                    error!("Failed to set TCP_NODELAY: {}", e);
                }

                // Create a buffer for reading the incoming message
                let mut buffer = [0u8; 4096];

                // Read from the socket into our buffer
                match socket.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        // Convert the received bytes to a string
                        if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                            // Look for the message type tag (35=X)
                            if let Some(msg_type) = extract_message_type(&message) {
                                // Generate appropriate response based on message type
                                let response = match MessageType::from_fix(msg_type) {
                                    Some(MessageType::Logon) | Some(MessageType::Logout) => {
                                        "Session Functionality coming soon\n"
                                    }
                                    Some(MessageType::NewOrderSingle) |
                                    Some(MessageType::MarketDataRequest) |
                                    Some(MessageType::MarketDataSnapshot) => {
                                        "Once we have sessions up and running we'll implement this\n"
                                    }
                                    Some(MessageType::Heartbeat) => {
                                        "Heartbeat received\n"
                                    }
                                    None => "Unsupported message type\n"
                                };

                                // Send the response back to the client
                                if let Err(e) = socket.write_all(response.as_bytes()).await {
                                    error!("Failed to send response: {}", e);
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        info!("Connection closed by client: {}", addr);
                    }
                    Err(e) => {
                        error!("Failed to read from socket: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

// Helper function to extract the message type from a FIX message
fn extract_message_type(message: &str) -> Option<&str> {
    // Look for the message type tag (35=X)
    message.split('|')
        .find(|field| field.starts_with("35="))
        .map(|field| &field[3..])
}