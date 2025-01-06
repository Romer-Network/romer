mod cmd;
mod config;
mod identity;
mod node;

use std::process;

use clap::Parser;
use tracing::{error, info, warn};

use crate::cmd::cli::NodeCliArgs;
use crate::identity::keymanager::NodeKeyManager;
use crate::node::hardware_validator::{HardwareDetector, VirtualizationType};
use crate::node::location_validator::LocationValidator;

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
    let location_verifier = LocationValidator::new();

    // Gold Coast coordinates
    let lat = -28.0167;
    let lon = 153.4000;

    // Perform location validation
    let validation_result = location_verifier
        .validate_location(lat, lon)
        .await
        .map_err(|e| format!("Location validation error: {}", e))?;

    // Detailed analysis of the validation result
    if validation_result.is_valid {
        // Location verified successfully
        info!(
            "Location verification passed 
            - Confidence: {:.2}%", 
            validation_result.confidence * 100.0
        );

        // Log any minor inconsistencies
        if !validation_result.inconsistencies.is_empty() {
            warn!("Minor location inconsistencies detected:");
            for issue in &validation_result.inconsistencies {
                warn!("- {}", issue);
            }
        }

        Ok(())
    } else {
        // Location verification failed
        error!(
            "Location verification failed 
            - Confidence: {:.2}%
            - Detected inconsistencies:",
            validation_result.confidence * 100.0
        );

        // Detailed logging of inconsistencies
        for issue in &validation_result.inconsistencies {
            error!("- {}", issue);
        }

        // Return a specific error with confidence level
        Err(format!(
            "Location verification failed with {:.2}% confidence. {} inconsistencies detected.",
            validation_result.confidence * 100.0,
            validation_result.inconsistencies.len()
        ))
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

    info!("Verifying hardware requirements...");
    if let Err(e) = verify_hardware_requirements() {
        error!("Hardware verification failed: {}", e);
        process::exit(1);
    }
    info!("Hardware verification passed");

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
