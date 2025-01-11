// src/network/types.rs

use crate::session::state::{Session, SessionState};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use uuid::Uuid;
use thiserror::Error;

/// Represents a FIX connection with its associated session
pub struct Connection {
    /// Unique identifier for this connection
    pub connection_id: Uuid,
    /// The TCP stream for this connection
    pub stream: TcpStream,
    /// Remote address of the connection
    pub remote_addr: SocketAddr,
    /// Associated session ID if authenticated
    pub session_id: Option<Uuid>,
    /// Channel for sending messages to this connection
    pub message_tx: mpsc::Sender<OutgoingMessage>,
    /// Channel for receiving messages from this connection
    pub message_rx: mpsc::Receiver<IncomingMessage>,
    /// Last time activity was seen on this connection
    pub last_activity: std::time::Instant,
}

impl Connection {
    /// Create a new connection from a TCP stream
    pub fn new(stream: TcpStream, remote_addr: SocketAddr) -> (Self, mpsc::Sender<IncomingMessage>) {
        let connection_id = Uuid::new_v4();
        let (message_tx, rx) = mpsc::channel(100);
        let (tx, message_rx) = mpsc::channel(100);
        
        let connection = Self {
            connection_id,
            stream,
            remote_addr,
            session_id: None,
            message_tx,
            message_rx,
            last_activity: std::time::Instant::now(),
        };
        
        (connection, tx)
    }

    /// Update the last activity timestamp
    pub fn record_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    /// Check if the connection has been idle too long
    pub fn is_idle(&self, timeout: std::time::Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}

/// Message received from a connection
#[derive(Debug)]
pub struct IncomingMessage {
    /// ID of the connection that received this message
    pub connection_id: Uuid,
    /// Raw message bytes
    pub data: Vec<u8>,
    /// When the message was received
    pub received_at: std::time::Instant,
}

/// Message to be sent on a connection
#[derive(Debug)]
pub struct OutgoingMessage {
    /// ID of the connection to send this message on
    pub connection_id: Uuid,
    /// Message data to send
    pub data: Vec<u8>,
}

/// Statistics about network operations
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Number of messages received
    pub messages_received: u64,
    /// Number of messages sent
    pub messages_sent: u64,
    /// Number of bytes received
    pub bytes_received: u64,
    /// Number of bytes sent
    pub bytes_sent: u64,
    /// Number of failed connections
    pub failed_connections: u64,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            active_connections: 0,
            messages_received: 0,
            messages_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            failed_connections: 0,
        }
    }
}

/// Configuration for network operations
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Address to bind the server to
    pub bind_address: String,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Size of connection message buffers
    pub message_buffer_size: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection idle timeout
    pub idle_timeout: std::time::Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8585".to_string(),
            max_connections: 1000,
            message_buffer_size: 100,
            max_message_size: 4096,
            idle_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Errors that can occur during network operations
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,

    #[error("Connection not found: {0}")]
    ConnectionNotFound(Uuid),

    #[error("Message too large: {size} bytes")]
    MessageTooLarge { size: usize },

    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),
}

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpSocket;

    #[tokio::test]
    async fn test_connection_creation() {
        // Create a mock TCP connection
        let socket = TcpSocket::new_v4().unwrap();
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let stream = socket.connect(addr).await.unwrap();
        let remote_addr = stream.peer_addr().unwrap();

        // Create a new connection
        let (connection, _tx) = Connection::new(stream, remote_addr);

        assert!(connection.session_id.is_none());
        assert_eq!(connection.remote_addr, remote_addr);
    }

    #[test]
    fn test_idle_detection() {
        // Create a mock connection
        let socket = TcpSocket::new_v4().unwrap();
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let stream = socket.connect(addr).await.unwrap();
        let remote_addr = stream.peer_addr().unwrap();
        
        let (mut connection, _tx) = Connection::new(stream, remote_addr);

        // Should not be idle initially
        assert!(!connection.is_idle(std::time::Duration::from_secs(1)));

        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Should now be idle
        assert!(connection.is_idle(std::time::Duration::from_secs(1)));

        // Record activity
        connection.record_activity();

        // Should no longer be idle
        assert!(!connection.is_idle(std::time::Duration::from_secs(1)));
    }

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();
        
        assert_eq!(config.bind_address, "0.0.0.0:8585");
        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.message_buffer_size, 100);
        assert_eq!(config.max_message_size, 4096);
        assert_eq!(config.idle_timeout, std::time::Duration::from_secs(30));
    }
}