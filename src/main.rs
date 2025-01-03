// main.rs
mod block;
mod cmd;
mod config;
mod consensus;
mod identity;
mod node;
mod utils;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use commonware_cryptography::{Ed25519, PublicKey, Scheme};
use commonware_p2p::authenticated;
use commonware_runtime::deterministic::Executor as DeterministicExecutor;
use commonware_runtime::tokio::Executor as TokioExecutor;
use commonware_runtime::Runner;
use config::runtime::{RuntimeConfig, RuntimeEnvironment};
use identity::keymanager::KeyManagementError;
use prometheus_client::registry::Registry;
use tracing::{error, info};

use crate::cmd::cli::NodeCliArgs;
use crate::identity::keymanager::NodeKeyManager;
use crate::node::hardware_validator::{HardwareDetector, VirtualizationType};

const ROMER_NAMESPACE: &[u8] = b"ROMER";

// Hardware verification is now a standalone function at the top level
fn verify_hardware_requirements() -> Result<(), String> {
    match HardwareDetector::detect_virtualization() {
        Ok(virtualization_type) => match virtualization_type {
            VirtualizationType::Physical => {
                info!("Running on physical hardware - verification passed");
                Ok(())
            }
            VirtualizationType::Virtual(tech) => {
                error!("Node detected running in virtual environment: {}", tech);
                Err(format!(
                    "Node is not allowed to run in virtual environment: {}",
                    tech
                ))
            }
        },
        Err(e) => {
            error!("Failed to detect virtualization environment: {}", e);
            Err(format!("Hardware verification failed: {}", e))
        }
    }
}

fn configure_bootstrappers(
    bootstrapper_args: Option<&Vec<String>>,
) -> Vec<(PublicKey, SocketAddr)> {
    let mut bootstrapper_identities = Vec::new();

    // If no bootstrappers provided, return empty vec - node will start fresh
    let Some(bootstrappers) = bootstrapper_args else {
        return bootstrapper_identities;
    };

    for bootstrapper in bootstrappers {
        // Split the bootstrapper string on @ symbol
        let parts: Vec<&str> = bootstrapper.split('@').collect();

        // Validate format
        if parts.len() != 2 {
            error!(
                "Invalid bootstrapper format. Expected 'seed@ip:port', got: {}",
                bootstrapper
            );
            continue;
        }

        // Parse the seed and generate public key
        match parts[0].parse::<u64>() {
            Ok(seed) => {
                let verifier = Ed25519::from_seed(seed).public_key();

                // Parse the socket address
                match SocketAddr::from_str(parts[1]) {
                    Ok(addr) => {
                        bootstrapper_identities.push((verifier, addr));
                        info!("Added bootstrapper: {}@{}", seed, addr);
                    }
                    Err(e) => {
                        error!("Invalid bootstrapper address {}: {}", parts[1], e);
                        continue;
                    }
                }
            }
            Err(e) => {
                error!("Invalid bootstrapper seed {}: {}", parts[0], e);
                continue;
            }
        }
    }

    bootstrapper_identities
}

fn main() {
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

    // Verify hardware requirements first
    if let Err(e) = verify_hardware_requirements() {
        error!("Hardware verification failed: {}", e);
        process::exit(1);
    }

    let args: NodeCliArgs = NodeCliArgs::parse();

    let mut runtime_config = match RuntimeConfig::load_default() {
        Ok(config) => {
            info!("Commonware Runtime configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load Commonware runtime configuration: {}", e);
            process::exit(1);
        }
    };

    // Override environment from config with CLI args if provided
    // Override config environment with CLI environment if provided
    runtime_config.environment = args.environment;

    // Initialize logging based on runtime environment
    tracing_subscriber::fmt()
        .with_max_level(match runtime_config.environment {
            RuntimeEnvironment::Development => tracing::Level::DEBUG,
            RuntimeEnvironment::Production => tracing::Level::INFO,
        })
        .with_target(true)
        .init();

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

            process::exit(1);
        }
    };

    let bootstrapper_identities = configure_bootstrappers(args.bootstrappers.as_ref());

    match runtime_config.environment {
        RuntimeEnvironment::Development => {
            // Use deterministic runtime for development/testing
            let dev_config = runtime_config
                .development
                .as_ref()
                .expect("Development configuration must be present");

            let (executor, runtime, auditor) = DeterministicExecutor::seeded(dev_config.seed);

            // Configure P2P with aggressive settings for development
            let p2p_cfg = authenticated::Config::aggressive(
                signer.clone(),
                ROMER_NAMESPACE,
                Arc::new(Mutex::new(Registry::default())),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), args.port),
                bootstrapper_identities,
                1024 * 1024,
            );

            // Development runtime initialization continues...
        }
        RuntimeEnvironment::Production => {
            // Use Tokio runtime for production deployment
            let prod_config = runtime_config
                .production
                .as_ref()
                .expect("Production configuration must be present");

            // Convert RuntimeConfig to Commonware Tokio Config
            let commonware_config = commonware_runtime::tokio::Config {
                registry: Arc::new(Mutex::new(Registry::default())),
                threads: prod_config.threads,
                catch_panics: prod_config.catch_panics,
                tcp_nodelay: Some(prod_config.tcp_nodelay),
                storage_directory: prod_config.storage_directory.clone(),
                read_timeout: Duration::from_millis(prod_config.read_timeout as u64),
                write_timeout: Duration::from_millis(prod_config.write_timeout as u64),
                maximum_buffer_size: prod_config.maximum_buffer_size,
            };

            let (executor, runtime_context) = TokioExecutor::init(commonware_config);

            // Configure P2P with recommended (conservative) settings for production
            let p2p_cfg = authenticated::Config::recommended(
                signer.clone(),
                ROMER_NAMESPACE,
                Arc::new(Mutex::new(Registry::default())),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), args.port),
                bootstrapper_identities,
                1024 * 1024, // 1MB max message size
            );

            // Production runtime initialization continues...
        }
    }
}
