// main.rs
mod block;
mod cmd;
mod config;
mod consensus;
mod identity;
mod node;
mod utils;

use clap::Parser;
use commonware_runtime::deterministic::Executor;
use commonware_runtime::Runner;
use identity::keymanager::KeyManagementError;
use node::validator::NodeError;
use tracing::{error, info};

use crate::cmd::cli::NodeCliArgs;
use crate::config::shared::SharedConfig;
use crate::identity::keymanager::NodeKeyManager;
use crate::node::validator::Node;

fn main() {
    // Parse command line arguments
    let args: NodeCliArgs = NodeCliArgs::parse();

    // Initialize logging with configured level
    tracing_subscriber::fmt()
        .with_max_level(args.get_log_level())
        .with_target(true)
        .init();

    let romer_ascii = r#"
    ██████╗  ██████╗ ███╗   ███╗███████╗██████╗ 
    ██╔══██╗██╔═══██╗████╗ ████║██╔════╝██╔══██╗
    ██████╔╝██║   ██║██╔████╔██║█████╗  ██████╔╝
    ██╔══██╗██║   ██║██║╚██╔╝██║██╔══╝  ██╔══██╗
    ██║  ██║╚██████╔╝██║ ╚═╝ ██║███████╗██║  ██║
    ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝
    "#;

    println!("{}", romer_ascii);

    info!("Starting Rømer Chain Node");

    // Load shared configuration
    let config = match SharedConfig::load_default() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            // Log additional details about which part of config failed
            error!("Configuration error details: {:?}", e);
            std::process::exit(1);
        }
    };

    // Initialize the key manager and get the signer in one step
    let signer = match NodeKeyManager::new().and_then(|km| km.initialize()) {
        Ok(signer) => signer,
        Err(e) => {
            error!("Failed to initialize key manager: {}", e);
            error!("Full error details: {:?}", e);

            if let KeyManagementError::Io(io_err) = &e {
                error!("IO Error details: {}", io_err);
                error!("Error kind: {:?}", io_err.kind());
            }

            std::process::exit(1);
        }
    };

    // Initialize the Commonware Runtime
    let (executor, runtime, _) = Executor::default();
    info!("Default Commonware Runtime initialized");

    // Create and run the node with configurations
    info!("Starting Node initialization...");

    Runner::start(executor, async move {
        let node = match Node::new(runtime.clone(), config, signer) {
            Ok(node) => {
                info!("Node successfully initialized");
                node
            }
            Err(e) => {
                error!("Failed to initialize node: {}", e);

                match e {
                    NodeError::Configuration(config_error) => {
                        // Log detailed configuration error information
                        error!("Configuration error occurred during node initialization");
                        if !config_error
                            .genesis_config_error
                            .to_string()
                            .contains("NotInitialized")
                        {
                            error!("Genesis error: {}", config_error.genesis_config_error);
                        }
                        if !config_error
                            .storage_config_error
                            .to_string()
                            .contains("NotInitialized")
                        {
                            error!("Storage error: {}", config_error.storage_config_error);
                        }
                        if !config_error
                            .tokenomics_config_error
                            .to_string()
                            .contains("NotInitialized")
                        {
                            error!("Tokenomics error: {}", config_error.tokenomics_config_error);
                        }
                    }
                    NodeError::Initialization(init_error) => {
                        error!("Node initialization failed: {}", init_error);
                        // Additional context about initialization failure
                        error!("Please check hardware requirements and network configuration");
                    }
                }
                // Exit with error code since we can't continue without a valid node
                std::process::exit(1);
            }
        };
        // Now run the node, handling any runtime errors
        if let Err(e) = node.run(args.address, args.get_bootstrap_addr()).await {
            error!("Node failed during operation: {}", e);
            // We might want to attempt recovery or cleanup here
            std::process::exit(1);
        }

        info!("Node shutting down gracefully");
    });
}
