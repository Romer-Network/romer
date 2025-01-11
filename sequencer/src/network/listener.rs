// src/network/listener.rs

use crate::network::types::{Connection, NetworkConfig, NetworkResult, NetworkError, NetworkStats};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn, error};

/// Control messages for the listener
#[derive(Debug, Clone)]
pub enum ListenerControl {
    /// Pause accepting new connections
    Pause,
    /// Resume accepting new connections
    Resume,
    /// Shutdown the listener
    Shutdown,
}

/// Manages TCP connection acceptance
pub struct ConnectionListener {
    /// Server configuration
    config: NetworkConfig,
    /// Current server statistics
    stats: Arc<RwLock<NetworkStats>>,
    /// Channel for new connection notifications
    connection_tx: mpsc::Sender<Connection>,
    /// Channel for control messages
    control_rx: broadcast::Receiver<ListenerControl>,
    /// Whether we're currently accepting connections
    accepting: Arc<RwLock<bool>>,
}

impl ConnectionListener {
    /// Create a new connection listener
    pub fn new(
        config: NetworkConfig,
        connection_tx: mpsc::Sender<Connection>,
        control_rx: broadcast::Receiver<ListenerControl>,
    ) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            connection_tx,
            control_rx,
            accepting: Arc::new(RwLock::new(true)),
        }
    }

    /// Start accepting connections
    pub async fn run(&mut self) -> NetworkResult<()> {
        // Bind to the configured address
        let listener = TcpListener::bind(&self.config.bind_address).await
            .map_err(NetworkError::ConnectionError)?;

        info!(
            address = %self.config.bind_address,
            "Connection listener started"
        );

        loop {
            // Check for control messages
            if let Ok(control) = self.control_rx.try_recv() {
                match control {
                    ListenerControl::Pause => {
                        *self.accepting.write() = false;
                        info!("Connection acceptance paused");
                        continue;
                    }
                    ListenerControl::Resume => {
                        *self.accepting.write() = true;
                        info!("Connection acceptance resumed");
                    }
                    ListenerControl::Shutdown => {
                        info!("Connection listener shutting down");
                        break;
                    }
                }
            }

            // Only accept if we're in accepting state
            if !*self.accepting.read() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            // Accept new connection
            let accept_result = tokio::select! {
                result = listener.accept() => result,
                _ = self.control_rx.recv() => continue,
            };

            match accept_result {
                Ok((stream, addr)) => {
                    // Check connection limit
                    let current_connections = self.stats.read().active_connections;
                    if current_connections >= self.config.max_connections {
                        warn!(
                            remote = %addr,
                            current = current_connections,
                            max = self.config.max_connections,
                            "Connection limit exceeded, rejecting connection"
                        );
                        self.stats.write().failed_connections += 1;
                        continue;
                    }

                    // Configure the TCP stream
                    if let Err(e) = self.configure_stream(&stream) {
                        error!(
                            remote = %addr,
                            error = %e,
                            "Failed to configure connection"
                        );
                        self.stats.write().failed_connections += 1;
                        continue;
                    }

                    // Create new connection
                    let (connection, _) = Connection::new(stream, addr);
                    let connection_id = connection.connection_id;

                    // Send to connection manager
                    if let Err(e) = self.connection_tx.send(connection).await {
                        error!(
                            connection_id = %connection_id,
                            error = %e,
                            "Failed to send connection to manager"
                        );
                        self.stats.write().failed_connections += 1;
                        continue;
                    }

                    // Update statistics
                    let mut stats = self.stats.write();
                    stats.active_connections += 1;

                    info!(
                        connection_id = %connection_id,
                        remote = %addr,
                        active = stats.active_connections,
                        "New connection accepted"
                    );
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "Failed to accept connection"
                    );
                    self.stats.write().failed_connections += 1;
                }
            }
        }

        Ok(())
    }

    /// Configure TCP stream options
    fn configure_stream(&self, stream: &tokio::net::TcpStream) -> NetworkResult<()> {
        // Set TCP_NODELAY to reduce latency
        stream.set_nodelay(true)
            .map_err(NetworkError::ConnectionError)?;

        // Set keep-alive to detect dead connections
        stream.set_keepalive(Some(std::time::Duration::from_secs(60)))
            .map_err(NetworkError::ConnectionError)?;

        Ok(())
    }

    /// Get current listener statistics
    pub fn get_stats(&self) -> NetworkStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpSocket;

    async fn create_test_listener() -> (ConnectionListener, broadcast::Sender<ListenerControl>) {
        // Create channels
        let (connection_tx, _) = mpsc::channel(10);
        let (control_tx, control_rx) = broadcast::channel(10);

        // Create config with random available port
        let mut config = NetworkConfig::default();
        config.bind_address = "127.0.0.1:0".to_string();

        let listener = ConnectionListener::new(
            config,
            connection_tx,
            control_rx,
        );

        (listener, control_tx)
    }

    #[tokio::test]
    async fn test_listener_lifecycle() {
        let (mut listener, control_tx) = create_test_listener().await;

        // Start listener in background
        let handle = tokio::spawn(async move {
            listener.run().await.unwrap();
        });

        // Send shutdown signal
        control_tx.send(ListenerControl::Shutdown).unwrap();

        // Wait for shutdown
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_connection_acceptance() {
        let (mut listener, _) = create_test_listener().await;

        // Start listener in background
        let handle = tokio::spawn(async move {
            listener.run().await.unwrap();
        });

        // Create test connection
        let socket = TcpSocket::new_v4().unwrap();
        let addr = listener.config.bind_address.parse().unwrap();
        let _stream = socket.connect(addr).await.unwrap();

        // Check stats
        assert_eq!(listener.get_stats().active_connections, 1);

        handle.abort();
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let (mut listener, control_tx) = create_test_listener().await;

        // Start listener in background
        let handle = tokio::spawn(async move {
            listener.run().await.unwrap();
        });

        // Pause acceptance
        control_tx.send(ListenerControl::Pause).unwrap();

        // Try connection - should fail
        let socket = TcpSocket::new_v4().unwrap();
        let addr = listener.config.bind_address.parse().unwrap();
        let result = socket.connect(addr).await;
        assert!(result.is_err());

        // Resume acceptance
        control_tx.send(ListenerControl::Resume).unwrap();

        // Try connection again - should succeed
        let socket = TcpSocket::new_v4().unwrap();
        let _stream = socket.connect(addr).await.unwrap();

        handle.abort();
    }
}