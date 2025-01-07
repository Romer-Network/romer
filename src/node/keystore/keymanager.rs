use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tracing::info;

use commonware_cryptography::{Ed25519, PrivateKey, Scheme};
use rand::rngs::OsRng;

// Import the hardware detector for OS detection
use crate::validation::hardware_validator::{HardwareDetector, OperatingSystem};

/// Manages node key generation, storage, and retrieval across different platforms
pub struct NodeKeyManager {
    /// Path where the node's private key is stored
    key_dir: PathBuf,

    /// Detected operating system to enable platform-specific handling
    os: OperatingSystem,
}

impl NodeKeyManager {
    /// Creates a new NodeKeyManager, detecting the appropriate key storage location
    /// based on the current operating system
    pub fn new() -> Result<Self> {
        // Detect the current operating system
        let os = HardwareDetector::detect_os();

        // Determine the appropriate key storage directory based on OS
        let key_dir = match os {
            OperatingSystem::Windows => {
                // Windows-specific path using USERPROFILE environment variable
                let user_profile = std::env::var("USERPROFILE")
                    .context("Could not find Windows user profile directory")?;
                PathBuf::from(user_profile).join(".romer")
            }
            OperatingSystem::MacOS | OperatingSystem::Linux => {
                let home_dir = dirs::home_dir().context("Could not find user home directory")?;

                info!("Home directory path: {:?}", home_dir);
                let key_dir = home_dir.join(".romer");
                info!("Constructed key directory path: {:?}", key_dir);

                fs::create_dir_all(&key_dir)
                    .with_context(|| format!("Failed to create directory at {:?}", key_dir))?;

                info!("Successfully created/verified .romer directory");
                key_dir
            }
            OperatingSystem::Unknown => {
                // Fallback to current directory for unknown systems
                let current_dir = std::env::current_dir()?;
                info!("Using current directory for key storage: {:?}", current_dir);
                current_dir.join(".romer")
            }
        };

        // Ensure the directory exists (additional check)
        fs::create_dir_all(&key_dir)?;

        Ok(Self {
            key_dir,
            os, // Store the detected OS for potential future use
        })
    }

    /// Initializes the node key, either loading an existing key or generating a new one
    pub fn initialize(&self, node_id: &str) -> Result<Ed25519> {
        info!("Initializing node key manager for {:?}", self.os);

        // Check for existing key and handle key generation in one flow
        let signer = match self.check_existing_key(node_id)? {
            Some(existing_key) => {
                info!("Loaded existing validator key for node {}", node_id);
                existing_key
            }
            None => {
                info!("No existing key found for node {}, generating new validator key", node_id);
                self.generate_key(node_id)?
            }
        };

        // Log key information for debugging and verification
        info!("Validator key ready for node {}", node_id);
        info!("Public key: {}", hex::encode(signer.public_key()));
        info!("Key stored at: {:?}", self.key_path(node_id));

        Ok(signer)
    }

    /// Generates a new cryptographic key and saves it to the key file
    pub fn generate_key(&self, node_id: &str) -> Result<Ed25519> {
        // Generate a new cryptographic key using the operating system's random number generator
        let signer = Ed25519::new(&mut OsRng);

        // Save the generated key to the specified path
        self.save_key(node_id, &signer)?;

        Ok(signer)
    }

    fn save_key(&self, node_id: &str, signer: &Ed25519) -> Result<()> {
        let private_key_bytes = signer.private_key();
        let key_path = self.key_path(node_id);

        if let Some(parent_dir) = key_path.parent() {
            fs::create_dir_all(parent_dir)
                .with_context(|| format!("Failed to create directory at {:?}", parent_dir))?;
        }

        fs::write(&key_path, private_key_bytes)
            .with_context(|| format!("Failed to write key file at {:?}", key_path))?;

        info!("Successfully wrote key for node {} to path: {:?}", node_id, key_path);
        Ok(())
    }

    /// Checks for an existing key file and attempts to load it
    pub fn check_existing_key(&self, node_id: &str) -> Result<Option<Ed25519>> {
        let key_path = self.key_path(node_id);
        if !key_path.exists() {
            return Ok(None);
        }

        let key_bytes = fs::read(&key_path)
            .with_context(|| format!("Failed to read key file at {:?}", key_path))?;

        if key_bytes.is_empty() {
            return Err(anyhow::anyhow!("Empty key file"));
        }

        let private_key = PrivateKey::try_from(key_bytes).context("Invalid key format")?;

        <Ed25519 as Scheme>::from(private_key)
            .ok_or_else(|| anyhow::anyhow!("Failed to reconstruct key"))
            .map(Some)
    }

    /// Retrieves the key path for the given node ID
    fn key_path(&self, node_id: &str) -> PathBuf {
        self.key_dir.join(format!("node_{}.key", node_id))
    }

    /// Retrieves detailed signer information for logging or display
    pub fn get_signer_info(&self, node_id: &str, signer: &Ed25519) -> (String, PathBuf) {
        (hex::encode(signer.public_key()), self.key_path(node_id))
    }

    /// Returns the detected operating system
    pub fn get_os(&self) -> &OperatingSystem {
        &self.os
    }
}