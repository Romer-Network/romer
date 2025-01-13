// use crate::network::manager::NetworkManager;
// use crate::network::types::{NetworkConfig, NetworkError, NetworkResult};
// use crate::session::manager::SessionManager;
// use crate::session::auth::SessionAuthenticator;
//use crate::fix::parser::FixParser;
use crate::fix::types::{FixConfig, ValidatedMessage};
// use crate::block::batch::BatchManager;
// use crate::block::builder::BlockBuilder;
// use crate::block::timer::BlockTimer;
// use network::types::NetworkStats;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;
use std::sync::Arc;
use thiserror::Error;

// Declare our module structure
// mod session;
mod fix;
// mod block;
// mod network;

fn main () {
    print!("Coming Soon!");
}

/*  
/// Errors that can occur during sequencer operation
#[derive(Error, Debug)]
pub enum SequencerError {
    #[error("Network error: {0}")]
    NetworkError(#[from] NetworkError),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Configuration for the sequencer
pub struct SequencerConfig {
    /// Maximum messages per block
    pub max_messages: usize,
    /// Maximum time to wait for a block
    pub block_window: Duration,
    /// Channel buffer sizes
    pub channel_size: usize,
    /// Network configuration
    pub network: NetworkConfig,
    /// FIX protocol configuration
    pub fix: FixConfig,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            max_messages: 500,
            block_window: Duration::from_millis(500),
            channel_size: 1000,
            network: NetworkConfig::default(),
            fix: FixConfig::default(),
        }
    }
}

/// Main sequencer application that coordinates all components
pub struct Sequencer {
    /// Configuration
    config: SequencerConfig,
    /// Network manager for handling TCP connections
    network_manager: Arc<NetworkManager>,
    /// Session manager for FIX session handling
    session_manager: Arc<SessionManager>,
    /// Authentication handler
    authenticator: Arc<SessionAuthenticator>,
    /// FIX message parser
    fix_parser: Arc<FixParser>,
    /// Batch manager for collecting messages
    batch_manager: Arc<BatchManager>,
    /// Block builder for creating blocks
    block_builder: Arc<BlockBuilder>,
    /// Block timer for controlling block creation
    block_timer: Arc<BlockTimer>,
    /// Channel for shutting down components
    shutdown_tx: mpsc::Sender<()>,
}

impl Sequencer {
    /// Create and initialize a new sequencer with all components
    pub async fn new(config: SequencerConfig) -> Result<Self, SequencerError> {
        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        // Create channels for component communication
        let (raw_message_tx, mut raw_message_rx) = mpsc::channel(config.channel_size);
        let (validated_tx, validated_rx) = mpsc::channel(config.channel_size);
        let (batch_tx, batch_rx) = mpsc::channel(config.channel_size);
        let (block_tx, block_rx) = mpsc::channel(config.channel_size);
        let (timer_tx, timer_rx) = mpsc::channel(config.channel_size);

        // Initialize network manager first - this opens our listening socket
        let network_manager = Arc::new(
            NetworkManager::new(
                config.network.clone(),
                raw_message_tx,
            ).map_err(SequencerError::NetworkError)?
        );

        // Initialize all other components
        let session_manager = Arc::new(SessionManager::new(validated_tx));
        let authenticator = Arc::new(SessionAuthenticator::new());
        let fix_parser = Arc::new(FixParser::new());
        let batch_manager = Arc::new(BatchManager::new(
            batch_tx,
            config.max_messages,
            config.block_window,
        ));
        let block_builder = Arc::new(BlockBuilder::new());
        let block_timer = Arc::new(BlockTimer::new(timer_tx, config.block_window));

        Ok(Self {
            config,
            network_manager,
            session_manager,
            authenticator,
            fix_parser,
            batch_manager,
            block_builder,
            block_timer,
            shutdown_tx,
        })
    }

    /// Start all sequencer components and begin processing
    pub async fn run(&self) -> Result<(), SequencerError> {
        info!("Starting sequencer components...");

        // Clone Arc references for task handlers
        let network_manager = self.network_manager.clone();
        let session_manager = self.session_manager.clone();
        let batch_manager = self.batch_manager.clone();
        let block_timer = self.block_timer.clone();
        let fix_parser = self.fix_parser.clone();

        // Start network manager to accept connections
        let network_handle = tokio::spawn(async move {
            info!("Starting network manager");
            if let Err(e) = network_manager.run().await {
                error!(error = %e, "Network manager error");
            }
        });

        // Start session management
        let session_handle = tokio::spawn(async move {
            info!("Starting session manager");
            if let Err(e) = session_manager.run().await {
                error!(error = %e, "Session manager error");
            }
        });

        // Start batch management
        let batch_handle = tokio::spawn(async move {
            info!("Starting batch manager");
            if let Err(e) = batch_manager.run().await {
                error!(error = %e, "Batch manager error");
            }
        });

        // Start block timer
        let timer_handle = tokio::spawn(async move {
            info!("Starting block timer");
            if let Err(e) = block_timer.run().await {
                error!(error = %e, "Block timer error");
            }
        });

        info!(
            address = %self.config.network.bind_address,
            "Sequencer startup complete, accepting FIX connections"
        );

        // Wait for shutdown signal
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received, stopping sequencer...");
                self.shutdown().await?;
            }
            Err(e) => {
                error!(error = %e, "Error waiting for shutdown signal");
                self.shutdown().await?;
            }
        }

        // Wait for all tasks to complete
        let _ = tokio::try_join!(
            network_handle,
            session_handle,
            batch_handle,
            timer_handle,
        );

        info!("Sequencer shutdown complete");
        Ok(())
    }

    /// Initiate graceful shutdown of all components
    async fn shutdown(&self) -> Result<(), SequencerError> {
        info!("Initiating sequencer shutdown...");

        // Stop accepting new connections
        self.network_manager.shutdown().await
            .map_err(SequencerError::NetworkError)?;

        // Signal all components to shut down
        if let Err(e) = self.shutdown_tx.send(()).await {
            warn!(error = %e, "Error sending shutdown signal");
        }

        Ok(())
    }

    /// Register a new market maker
    pub async fn register_market_maker(
        &self,
        sender_comp_id: String,
        public_key: Vec<u8>,
    ) -> Result<Uuid, SequencerError> {
        // Register the public key
        self.authenticator.register_key(sender_comp_id.clone(), &public_key)
            .map_err(|e| SequencerError::SessionError(e.to_string()))?;
        
        // Create a new session
        let session_id = self.session_manager.create_session(
            sender_comp_id,
            "ROMER".to_string(), // Our standard target comp ID
            30, // Standard 30 second heartbeat
            public_key,
        ).map_err(|e| SequencerError::SessionError(e.to_string()))?;

        Ok(session_id)
    }

    /// Get current sequencer statistics
    pub fn get_stats(&self) -> SequencerStats {
        SequencerStats {
            network_stats: self.network_manager.get_stats(),
            active_sessions: self.session_manager.active_session_count(),
            blocks_created: self.block_builder.block_count(),
        }
    }
}

/// Statistics about sequencer operation
#[derive(Debug, Clone)]
pub struct SequencerStats {
    /// Network-related statistics
    pub network_stats: NetworkStats,
    /// Number of active FIX sessions
    pub active_sessions: usize,
    /// Total number of blocks created
    pub blocks_created: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with reasonable defaults
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Initializing Rømer Chain sequencer...");
    
    // Create default configuration
    let config = SequencerConfig::default();

    // Create and start the sequencer
    let sequencer = Sequencer::new(config).await?;
    sequencer.run().await?;

    Ok(())
}

    */