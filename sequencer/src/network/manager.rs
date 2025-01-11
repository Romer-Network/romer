// src/network/manager.rs

use crate::network::types::{Connection, NetworkConfig, NetworkStats, NetworkError, NetworkResult};
use crate::network::listener::{ConnectionListener, ListenerControl};
use crate::network::connection::ConnectionHandler;
use tokio::sync::{mpsc, broadcast};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error, debug};

/// Manages all network operations and connections
pub struct NetworkManager {
    /// Configuration settings
    config: NetworkConfig,
    /// Active connections by ID
    connections: Arc<RwLock<HashMap<Uuid, Connection>>>,
    /// Network statistics
    stats: Arc<RwLock<NetworkStats>>,
    /// Channel for new connections from listener
    connection_rx: mpsc::Receiver<Connection>,
    /// Channel for sending listener control messages
    listener_tx: broadcast::Sender<ListenerControl>,
    /// Channel for processed messages
    message_tx: mpsc::Sender<IncomingMessage>,
    /// Health check interval in seconds
    health_check_interval: u64,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(
        config: NetworkConfig,
        message_tx: mpsc::Sender<IncomingMessage>,
    ) -> NetworkResult<Self> {
        // Create channels
        let (connection_tx, connection_rx) = mpsc::channel(100);
        let (listener_tx, _) = broadcast::channel(10);

        // Create listener
        let mut listener = ConnectionListener::new(
            config.clone(),
            connection_tx,
            listener_tx.subscribe(),
        );

        // Start listener in background
        tokio::spawn(async move {
            if let Err(e) = listener.run().await {
                error!(error = %e, "Listener error");
            }
        });

        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            connection_rx,
            listener_tx,
            message_tx,
            health_check_interval: 30,
        })
    }

    /// Start the network manager
    pub async fn run(&mut self) -> NetworkResult<()> {
        info!("Starting network manager");

        // Start health check timer
        let health_check_interval = tokio::time::Duration::from_secs(self.health_check_interval);
        let mut health_check = tokio::time::interval(health_check_interval);

        loop {
            tokio::select! {
                // Handle new connections
                Some(connection) = self.connection_rx.recv() => {
                    self.handle_new_connection(connection).await?;
                }

                // Periodic health check
                _ = health_check.tick() => {
                    self.check_connection_health().await;
                }
            }
        }
    }

    /// Handle a new incoming connection
    async fn handle_new_connection(&mut self, connection: Connection) -> NetworkResult<()> {
        let connection_id = connection.connection_id;
        let remote_addr = connection.remote_addr;

        // Store connection
        self.connections.write().insert(connection_id, connection.clone());

        // Create message channels
        let (message_tx, message_rx) = mpsc::channel(self.config.message_buffer_size);

        // Create connection handler
        let mut handler = ConnectionHandler::new(
            connection,
            message_tx,
        );

        // Start handler in background
        let connections = self.connections.clone();
        let stats = self.stats.clone();
        tokio::spawn(async move {
            debug!(
                connection_id = %connection_id,
                remote = %remote_addr,
                "Starting connection handler"
            );

            // Run the handler
            if let Err(e) = handler.run().await {
                error!(
                    connection_id = %connection_id,
                    error = %e,
                    "Connection handler error"
                );
            }

            // Clean up connection
            connections.write().remove(&connection_id);
            stats.write().active_connections -= 1;

            debug!(
                connection_id = %connection_id,
                "Connection handler stopped"
            );
        });

        // Update statistics
        self.stats.write().active_connections += 1;

        info!(
            connection_id = %connection_id,
            remote = %remote_addr,
            "New connection initialized"
        );

        Ok(())
    }

    /// Check health of all connections
    async fn check_connection_health(&self) {
        let mut to_remove = Vec::new();

        // Check each connection
        for (id, conn) in self.connections.read().iter() {
            // Check if connection is idle
            if conn.is_idle(self.config.idle_timeout) {
                warn!(
                    connection_id = %id,
                    remote = %conn.remote_addr,
                    "Connection idle timeout"
                );
                to_remove.push(*id);
                continue;
            }

            // Add other health checks as needed
        }

        // Remove dead connections
        if !to_remove.is_empty() {
            let mut connections = self.connections.write();
            let mut stats = self.stats.write();
            
            for id in to_remove {
                connections.remove(&id);
                stats.active_connections -= 1;
            }
        }
    }

    /// Pause accepting new connections
    pub fn pause(&self) -> NetworkResult<()> {
        self.listener_tx.send(ListenerControl::Pause)
            .map_err(|e| NetworkError::SendError(e.to_string()))?;
        info!("Network manager paused");
        Ok(())
    }

    /// Resume accepting connections
    pub fn resume(&self) -> NetworkResult<()> {
        self.listener_tx.send(ListenerControl::Resume)
            .map_err(|e| NetworkError::SendError(e.to_string()))?;
        info!("Network manager resumed");
        Ok(())
    }

    /// Gracefully shutdown the network manager
    pub async fn shutdown(&self) -> NetworkResult<()> {
        info!("Starting network manager shutdown");

        // Stop accepting new connections
        self.listener_tx.send(ListenerControl::Shutdown)
            .map_err(|e| NetworkError::SendError(e.to_string()))?;

        // Close all active connections
        let connections = self.connections.read();
        for (id, _) in connections.iter() {
            if let Some(conn) = connections.get(id) {
                debug!(
                    connection_id = %id,
                    remote = %conn.remote_addr,
                    "Closing connection"
                );
            }
        }

        info!("Network manager shutdown complete");
        Ok(())
    }

    /// Get current statistics
    pub fn get_stats(&self) -> NetworkStats {
        self.stats.read().clone()
    }

    /// Get information about a specific connection
    pub fn get_connection(&self, id: Uuid) -> Option<Connection> {
        self.connections.read().get(&id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::net::TcpSocket;

    async fn create_test_manager() -> NetworkManager {
        let mut config = NetworkConfig::default();
        config.bind_address = "127.0.0.1:0".to_string();
        
        let (tx, _) = mpsc::channel(10);
        NetworkManager::new(config, tx).unwrap()
    }

    #[tokio::test]
    async fn test_manager_lifecycle() {
        let mut manager = create_test_manager().await;

        // Start manager in background
        let handle = tokio::spawn(async move {
            manager.run().await.unwrap();
        });

        // Give it time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create test connection
        let socket = TcpSocket::new_v4().unwrap();
        let addr = manager.config.bind_address.parse().unwrap();
        let _stream = socket.connect(addr).await.unwrap();

        // Check statistics
        assert_eq!(manager.get_stats().active_connections, 1);

        handle.abort();
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let manager = create_test_manager().await;

        // Pause and resume
        manager.pause().unwrap();
        manager.resume().unwrap();

        // Try connection while paused
        manager.pause().unwrap();
        let socket = TcpSocket::new_v4().unwrap();
        let addr = manager.config.bind_address.parse().unwrap();
        let result = socket.connect(addr).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_health_check() {
        let mut manager = create_test_manager().await;
        
        // Reduce health check interval for testing
        manager.health_check_interval = 1;

        // Start manager
        let handle = tokio::spawn(async move {
            manager.run().await.unwrap();
        });

        // Create connection that will timeout
        let socket = TcpSocket::new_v4().unwrap();
        let addr = manager.config.bind_address.parse().unwrap();
        let _stream = socket.connect(addr).await.unwrap();

        // Wait for health check to remove idle connection
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Connection should be removed
        assert_eq!(manager.get_stats().active_connections, 0);

        handle.abort();
    }
}