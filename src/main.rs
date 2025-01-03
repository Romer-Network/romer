mod cmd;
mod config;
mod identity;
mod node;

use std::process;
use std::str::FromStr;
use std::net::{IpAddr, SocketAddr};
use geo::Point;

use clap::Parser;
use commonware_cryptography::{Ed25519, Scheme};
use tracing::{error, info};

use crate::cmd::cli::NodeCliArgs;
use crate::identity::keymanager::NodeKeyManager;
use crate::node::hardware_validator::{HardwareDetector, VirtualizationType};
use crate::node::location_validator::LocationValidator;

// Gold Coast, Australia coordinates
const VALIDATOR_LATITUDE: f64 = -28.0167;
const VALIDATOR_LONGITUDE: f64 = 153.4000;

/// Verifies that the node is running on physical hardware, not in a virtual environment.
/// This is crucial for the security of the network as virtual machines could be used
/// to fake geographic distribution.
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

/// Verifies the physical location of the node using network latency measurements.
/// Returns Ok if the measured location matches the claimed location within acceptable bounds.
async fn verify_physical_location() -> Result<(), String> {
    let validator = LocationValidator::new();
    
    // Gold Coast coordinates for initial testing
    let claimed_location = Point::new(VALIDATOR_LONGITUDE, VALIDATOR_LATITUDE);
    
    match validator.validate_location(VALIDATOR_LATITUDE, VALIDATOR_LONGITUDE).await {
        Ok(validation) => {
            if validation.is_valid {
                info!("Location verification passed with confidence: {:.2}%", 
                      validation.confidence * 100.0);
                Ok(())
            } else {
                let issues = validation.inconsistencies.join("\n- ");
                Err(format!("Location verification failed:\n- {}", issues))
            }
        }
        Err(e) => Err(format!("Location verification error: {}", e))
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging with a reasonable default level
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
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

    // Parse command line arguments
    let args = NodeCliArgs::parse();

    // Step 1: Verify hardware requirements
    info!("Verifying hardware requirements...");
    if let Err(e) = verify_hardware_requirements() {
        error!("Hardware verification failed: {}", e);
        process::exit(1);
    }
    info!("Hardware verification passed");

    // Step 2: Initialize the key manager and get the signer
    info!("Initializing node identity...");
    let signer = match NodeKeyManager::new().and_then(|km| km.initialize()) {
        Ok(signer) => {
            info!("Node identity initialized successfully");
            signer
        }
        Err(e) => {
            error!("Failed to initialize key manager: {}", e);
            process::exit(1);
        }
    };

    // Step 3: Verify physical location
    info!("Verifying physical location...");
    if let Err(e) = verify_physical_location().await {
        error!("Location verification failed: {}", e);
        process::exit(1);
    }
    info!("Location verification passed");

    info!("Node initialization complete");
}