use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use commonware_cryptography::{Ed25519, PrivateKey, Scheme};
use rand::rngs::OsRng;
use thiserror::Error;

// Import the hardware detector for OS detection
use crate::node::hardware_validator::{HardwareDetector, OperatingSystem};

/// Comprehensive error handling for key management operations
#[derive(Error, Debug)]
pub enum KeyManagementError {
    /// Represents IO-related errors during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Represents cryptography-related errors
    #[error("Cryptography error: {0}")]
    Crypto(String),

    /// Represents errors in home directory or user profile detection
    #[error("Directory access error: {0}")]
    DirectoryAccess(String),
}

/// Manages node key generation, storage, and retrieval across different platforms
pub struct NodeKeyManager {
    /// Path where the node's private key is stored
    key_path: PathBuf,

    /// Detected operating system to enable platform-specific handling
    os: OperatingSystem,
}

impl NodeKeyManager {
    /// Creates a new NodeKeyManager, detecting the appropriate key storage location
    /// based on the current operating system
    pub fn new() -> Result<Self, KeyManagementError> {
        // Detect the current operating system
        let os = HardwareDetector::detect_os();

        // Determine the appropriate key storage directory based on OS
        let key_dir = match os {
            OperatingSystem::Windows => {
                // Windows-specific path using USERPROFILE environment variable
                let user_profile = std::env::var("USERPROFILE").map_err(|_| {
                    KeyManagementError::DirectoryAccess(
                        "Could not find Windows user profile directory".to_string(),
                    )
                })?;
                PathBuf::from(user_profile).join(".romer")
            }
            OperatingSystem::MacOS | OperatingSystem::Linux => {
                // On Unix-like systems, use home directory
                let home_dir = dirs::home_dir().ok_or_else(|| {
                    KeyManagementError::DirectoryAccess(
                        "Could not find user home directory".to_string(),
                    )
                })?;

                // Add explicit logging and debugging
                info!("Home directory path: {:?}", home_dir);

                let key_dir = home_dir.join(".romer");

                info!("Constructed key directory path: {:?}", key_dir);

                // Check if the directory exists or can be created
                match fs::create_dir_all(&key_dir) {
                    Ok(_) => {
                        info!("Successfully created/verified .romer directory");
                        key_dir
                    }
                    Err(e) => {
                        error!("Failed to create .romer directory: {}", e);
                        return Err(KeyManagementError::Io(e));
                    }
                }
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

        // Set the full path for the key file
        let key_path = key_dir.join("node.key");

        info!("Final key path: {:?}", key_path);

        Ok(Self {
            key_path,
            os, // Store the detected OS for potential future use
        })
    }

    /// Initializes the node key, either loading an existing key or generating a new one
    pub fn initialize(&self) -> Result<Ed25519, KeyManagementError> {
        info!("Initializing node key manager for {:?}", self.os);

        // Check for existing key and handle key generation in one flow
        let signer = match self.check_existing_key()? {
            Some(existing_key) => {
                info!("Loaded existing validator key");
                existing_key
            }
            None => {
                info!("No existing key found, generating new validator key");
                self.generate_key()?
            }
        };

        // Log key information for debugging and verification
        info!("Validator key ready");
        info!("Public key: {}", hex::encode(signer.public_key()));
        info!("Key stored at: {:?}", self.key_path());

        Ok(signer)
    }

    /// Generates a new cryptographic key and saves it to the key file
    pub fn generate_key(&self) -> Result<Ed25519, KeyManagementError> {
        // Generate a new cryptographic key using the operating system's random number generator
        let signer = Ed25519::new(&mut OsRng);

        // Save the generated key to the specified path
        self.save_key(&signer)?;

        Ok(signer)
    }

    fn save_key(&self, signer: &Ed25519) -> Result<(), KeyManagementError> {
        // Retrieve the private key bytes
        let private_key_bytes = signer.private_key();

        // Ensure the parent directory exists
        if let Some(parent_dir) = self.key_path.parent() {
            fs::create_dir_all(parent_dir).map_err(|e| {
                error!("Failed to create parent directory: {}", e);
                KeyManagementError::Io(e)
            })?;
        }

        // Attempt to write the file with detailed logging
        match fs::write(&self.key_path, private_key_bytes) {
            Ok(_) => {
                info!("Successfully wrote key to path: {:?}", self.key_path);
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to write key file. Path: {:?}, Error: {}",
                    self.key_path, e
                );
                Err(KeyManagementError::Io(e))
            }
        }
    }

    /// Checks for an existing key file and attempts to load it
    pub fn check_existing_key(&self) -> Result<Option<Ed25519>, KeyManagementError> {
        // Check if key file exists
        if !self.key_path.exists() {
            return Ok(None);
        }

        // Read the entire file contents
        let key_bytes = std::fs::read(&self.key_path).map_err(|e| KeyManagementError::Io(e))?;

        // Validate key bytes
        if key_bytes.is_empty() {
            return Err(KeyManagementError::Crypto("Empty key file".to_string()));
        }

        // Attempt to reconstruct the private key
        let private_key = PrivateKey::try_from(key_bytes)
            .map_err(|e| KeyManagementError::Crypto(format!("Invalid key format: {}", e)))?;

        // Reconstruct the signer from the private key
        <Ed25519 as Scheme>::from(private_key)
            .ok_or_else(|| KeyManagementError::Crypto("Failed to reconstruct key".to_string()))
            .map(Some)
    }

    /// Retrieves the current key path
    pub fn key_path(&self) -> &PathBuf {
        &self.key_path
    }

    /// Retrieves detailed signer information for logging or display
    pub fn get_signer_info(&self, signer: &Ed25519) -> (String, &PathBuf) {
        (hex::encode(signer.public_key()), &self.key_path)
    }

    /// Returns the detected operating system
    pub fn get_os(&self) -> &OperatingSystem {
        &self.os
    }
}
