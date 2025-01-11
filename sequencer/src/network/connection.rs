// src/network/connection.rs

use crate::network::types::{Connection, IncomingMessage, OutgoingMessage, NetworkError, NetworkResult};
use crate::network::codec::FixCodec;
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use bytes::{BytesMut, BufMut};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, warn, error, debug};

/// Size of the TCP read buffer
const READ_BUFFER_SIZE: usize = 8192;

/// Manages an individual TCP connection
pub struct ConnectionHandler {
    /// The connection being handled
    connection: Connection,
    /// Buffer for incoming data
    read_buffer: BytesMut,
    /// Buffer for outgoing data
    write_buffer: BytesMut,
    /// FIX message codec
    codec: FixCodec,
    /// Channel for forwarding processed messages
    message_tx: mpsc::Sender<IncomingMessage>,
    /// Statistics for this connection
    stats: Arc<Mutex<ConnectionStats>>,
}

/// Statistics for a single connection
#[derive(Debug, Default)]
pub struct ConnectionStats {
    /// Number of messages received
    pub messages_received: u64,
    /// Number of messages sent
    pub messages_sent: u64,
    /// Number of bytes received
    pub bytes_received: u64,
    /// Number of bytes sent
    pub bytes_sent: u64,
    /// Number of framing errors detected
    pub framing_errors: u64,
    /// Number of parse errors
    pub parse_errors: u64,
}

impl ConnectionHandler {
    /// Create a new connection handler
    pub fn new(
        connection: Connection,
        message_tx: mpsc::Sender<IncomingMessage>,
    ) -> Self {
        Self {
            connection,
            read_buffer: BytesMut::with_capacity(READ_BUFFER_SIZE),
            write_buffer: BytesMut::with_capacity(READ_BUFFER_SIZE),
            codec: FixCodec::new(),
            message_tx,
            stats: Arc::new(Mutex::new(ConnectionStats::default())),
        }
    }

    /// Start processing the connection
    pub async fn run(&mut self) -> NetworkResult<()> {
        // Split the TCP stream
        let (read_half, write_half) = self.connection.stream.split();
        let mut reader = BufReader::new(read_half);
        let mut writer = BufWriter::new(write_half);

        // Create channel for coordinating read and write tasks
        let (write_tx, mut write_rx) = mpsc::channel(100);

        // Spawn read task
        let connection_id = self.connection.connection_id;
        let message_tx = self.message_tx.clone();
        let stats = self.stats.clone();
        let mut read_buffer = BytesMut::with_capacity(READ_BUFFER_SIZE);
        let read_task = tokio::spawn(async move {
            let mut tmp_buf = [0u8; READ_BUFFER_SIZE];
            
            loop {
                // Read from TCP stream
                match reader.read(&mut tmp_buf).await {
                    Ok(0) => {
                        // EOF - connection closed
                        break;
                    }
                    Ok(n) => {
                        // Update statistics
                        stats.lock().bytes_received += n as u64;

                        // Append to buffer
                        read_buffer.put_slice(&tmp_buf[..n]);

                        // Process complete messages
                        while let Some(msg) = FixCodec::try_parse(&mut read_buffer)? {
                            stats.lock().messages_received += 1;
                            
                            // Forward message
                            let incoming = IncomingMessage {
                                connection_id,
                                data: msg.to_vec(),
                                received_at: std::time::Instant::now(),
                            };
                            
                            if let Err(e) = message_tx.send(incoming).await {
                                error!(
                                    connection_id = %connection_id,
                                    error = %e,
                                    "Failed to forward message"
                                );
                                return Err(NetworkError::SendError(e.to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(NetworkError::ConnectionError(e));
                    }
                }
            }

            Ok(())
        });

        // Spawn write task
        let stats = self.stats.clone();
        let write_task = tokio::spawn(async move {
            let mut write_buffer = BytesMut::with_capacity(READ_BUFFER_SIZE);
            
            while let Some(msg) = write_rx.recv().await {
                // Add message to buffer
                write_buffer.put_slice(&msg.data);
                
                // Write to TCP stream
                match writer.write_all(&write_buffer).await {
                    Ok(_) => {
                        stats.lock().bytes_sent += write_buffer.len() as u64;
                        stats.lock().messages_sent += 1;
                        
                        // Clear buffer after successful write
                        write_buffer.clear();
                    }
                    Err(e) => {
                        return Err(NetworkError::ConnectionError(e));
                    }
                }
                
                // Ensure data is sent
                if let Err(e) = writer.flush().await {
                    return Err(NetworkError::ConnectionError(e));
                }
            }

            Ok(())
        });

        // Handle incoming messages from connection manager
        while let Some(message) = self.connection.message_rx.recv().await {
            if let Err(e) = write_tx.send(message).await {
                error!(
                    connection_id = %self.connection.connection_id,
                    error = %e,
                    "Failed to forward outgoing message"
                );
                break;
            }
        }

        // Wait for tasks to complete
        let (read_result, write_result) = tokio::join!(read_task, write_task);

        // Check for errors
        if let Err(e) = read_result {
            error!(
                connection_id = %self.connection.connection_id,
                error = %e,
                "Read task panicked"
            );
        }

        if let Err(e) = write_result {
            error!(
                connection_id = %self.connection.connection_id,
                error = %e,
                "Write task panicked"
            );
        }

        Ok(())
    }

    /// Get statistics for this connection
    pub fn get_stats(&self) -> ConnectionStats {
        self.stats.lock().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use std::net::SocketAddr;

    async fn create_test_connection() -> (ConnectionHandler, TcpStream) {
        // Create test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Create client connection
        let client = TcpStream::connect(addr).await.unwrap();
        let (server, _) = listener.accept().await.unwrap();

        // Create connection handler
        let (tx, _) = mpsc::channel(10);
        let connection = Connection::new(server, addr);
        let handler = ConnectionHandler::new(connection, tx);

        (handler, client)
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let (mut handler, client) = create_test_connection().await;

        // Start handler in background
        let handle = tokio::spawn(async move {
            handler.run().await.unwrap();
        });

        // Close client connection
        drop(client);

        // Wait for handler to finish
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_message_processing() {
        let (mut handler, mut client) = create_test_connection().await;

        // Start handler in background
        let handle = tokio::spawn(async move {
            handler.run().await.unwrap();
        });

        // Send test message
        let test_msg = b"8=FIX.4.2\x019=0\x0135=0\x0110=0\x01";
        client.write_all(test_msg).await.unwrap();

        // Wait a bit for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check statistics
        let stats = handler.get_stats();
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.bytes_received, test_msg.len() as u64);

        // Clean up
        drop(client);
        handle.await.unwrap();
    }
}