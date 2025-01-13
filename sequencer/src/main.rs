use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up basic logging so we can see what's happening
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    // Get configuration from environment variables or use defaults
    let host = std::env::var("SEQUENCER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SEQUENCER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9878);

    // Create the bind address
    let addr = format!("{}:{}", host, port);

    // Create a TCP listener bound to the specified address
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    // Accept new connections in a loop
    loop {
        // Wait for a new connection
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("Accepted connection from: {}", addr);

                // Enable TCP_NODELAY for better latency
                if let Err(e) = socket.set_nodelay(true) {
                    error!("Failed to set TCP_NODELAY: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
