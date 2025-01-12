use chrono::{DateTime, Duration, Utc};
use rand::rngs::OsRng;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::keymanager::{
    KeyManagerError, KeyManagerResult, SessionKeyData, SignatureScheme,
};
use crate::utils::hardware_validator::{HardwareDetector, OperatingSystem};
use commonware_cryptography::{Bls12381, Ed25519, PrivateKey, PublicKey, Scheme, Signature};

/// Manages cryptographic keys for the system, supporting both permanent and session keys.
/// Handles secure storage, session management, and key operations while maintaining
/// separation between storage format and cryptographic operations.
pub struct KeyManager {
    /// Base directory for key storage
    pub base_dir: PathBuf,
    /// Directory for permanent keys
    pub permanent_dir: PathBuf,
    /// Directory for session keys
    pub session_dir: PathBuf,
    /// Detected operating system
    os: OperatingSystem,
}

impl KeyManager {
    /// Creates a new KeyManager instance, initializing the necessary directory structure
    /// based on the detected operating system.
    pub fn new() -> KeyManagerResult<Self> {
        let os = HardwareDetector::detect_os();
        let base_dir = Self::determine_base_dir(&os)?;
        let permanent_dir = base_dir.join("permanent");
        let session_dir = base_dir.join("sessions");

        // Ensure our directory structure exists
        fs::create_dir_all(&permanent_dir)
            .map_err(|e| KeyManagerError::StorageError(e.to_string()))?;
        fs::create_dir_all(&session_dir)
            .map_err(|e| KeyManagerError::StorageError(e.to_string()))?;

        Ok(Self {
            base_dir,
            permanent_dir,
            session_dir,
            os,
        })
    }

    /// Initializes a new key for the specified signature scheme.
    /// Returns the public key bytes of the generated key.
    pub fn initialize(&self, scheme: SignatureScheme) -> KeyManagerResult<Vec<u8>> {
        match scheme {
            SignatureScheme::Ed25519 => {
                let signer = Ed25519::new(&mut OsRng);
                self.save_permanent_key(scheme, &signer.private_key().to_vec())?;
                Ok(signer.public_key().to_vec())
            }
            SignatureScheme::Bls12381 => {
                let signer = Bls12381::new(&mut OsRng);
                self.save_permanent_key(scheme, &signer.private_key().to_vec())?;
                Ok(signer.public_key().to_vec())
            }
        }
    }

    /// Creates a new session key signed by the specified permanent BLS key.
    /// The session key includes an expiration time and a specified purpose.
    pub fn create_session_key(
        &self,
        permanent_key_bytes: &[u8],
        namespace: &str,
        duration_hours: i64,
        purpose: &str,
    ) -> KeyManagerResult<SessionKeyData> {
        // Convert the permanent key bytes into a PrivateKey type
        let private_key = PrivateKey::from(permanent_key_bytes.to_vec());

        // Create the permanent key signer
        let mut permanent_key = <Bls12381 as Scheme>::from(private_key)
            .ok_or_else(|| KeyManagerError::InvalidKeyFormat("Invalid permanent key".into()))?;

        // Create a new session key
        let mut session_key = Bls12381::new(&mut OsRng);
        let session_key_bytes = session_key.private_key();

        let created_at = Utc::now();
        let expires_at = created_at + Duration::hours(duration_hours);

        // Create the message to sign, including all session key metadata
        let message = format!(
            "{}:{}:{}",
            hex::encode(session_key.public_key()),
            expires_at.timestamp(),
            purpose
        );

        // Sign using the provided namespace
        let namespace_bytes = namespace.as_bytes();
        let parent_signature = permanent_key.sign(namespace_bytes, message.as_bytes());

        let session_data = SessionKeyData {
            key_bytes: session_key_bytes.to_vec(),
            created_at,
            expires_at,
            parent_public_key: permanent_key.public_key().to_vec(),
            parent_signature: parent_signature.to_vec(),
            purpose: purpose.to_string(),
            namespace: namespace.to_string(),
        };

        self.save_session_key(&session_data)?;

        Ok(session_data)
    }

    /// Verifies a session key's validity
    pub fn verify_session_key(&self, session_data: &SessionKeyData) -> KeyManagerResult<bool> {
        // Check expiration first
        if Utc::now() > session_data.expires_at {
            return Err(KeyManagerError::SessionExpired);
        }

        // Convert the raw bytes into a PrivateKey type first
        let session_private_key = PrivateKey::from(session_data.key_bytes.clone());

        // Create a key instance from the session key bytes using the Scheme trait
        let session_key = <Bls12381 as Scheme>::from(session_private_key)
            .ok_or_else(|| KeyManagerError::InvalidKeyFormat("Invalid session key".into()))?;

        // Create the verification message
        let message = format!(
            "{}:{}:{}",
            hex::encode(session_key.public_key().to_vec()),
            session_data.expires_at.timestamp(),
            session_data.purpose
        );

        // For verification, we don't need to construct a full signer - we can use the static verify method
        let namespace_bytes = session_data.namespace.as_bytes();

        // Use the static verify method from the Scheme trait
        if !Bls12381::verify(
            namespace_bytes,
            message.as_bytes(),
            &PublicKey::from(session_data.parent_public_key.clone()),
            &Signature::from(session_data.parent_signature.clone()),
        ) {
            return Err(KeyManagerError::InvalidSessionSignature);
        }

        Ok(true)
    }

    /// Loads a permanent key of the specified scheme.
    /// Returns the key bytes which can be used to reconstruct the cryptographic type.
    pub fn load_permanent_key(&self, scheme: SignatureScheme) -> KeyManagerResult<Vec<u8>> {
        let path = self.get_permanent_key_path(scheme);
        if !path.exists() {
            return Err(KeyManagerError::KeyNotFound(format!(
                "No key found for scheme {:?}",
                scheme
            )));
        }

        fs::read(&path).map_err(|e| KeyManagerError::IoError(e))
    }

    /// Loads a session key by its identifier.
    pub fn load_session_key(&self, session_id: &str) -> KeyManagerResult<SessionKeyData> {
        let path = self.session_dir.join(format!("{}.json", session_id));
        if !path.exists() {
            return Err(KeyManagerError::KeyNotFound(format!(
                "Session key not found: {}",
                session_id
            )));
        }

        let content = fs::read_to_string(&path).map_err(|e| KeyManagerError::IoError(e))?;

        serde_json::from_str(&content)
            .map_err(|e| KeyManagerError::SerializationError(e.to_string()))
    }

    // Private helper methods

    /// Determines the appropriate base directory for key storage based on the operating system
    fn determine_base_dir(os: &OperatingSystem) -> KeyManagerResult<PathBuf> {
        let base = match os {
            OperatingSystem::Windows => {
                PathBuf::from(std::env::var("USERPROFILE").map_err(|_| {
                    KeyManagerError::InitializationError(
                        "Could not find Windows user profile".into(),
                    )
                })?)
            }
            OperatingSystem::MacOS | OperatingSystem::Linux => {
                dirs::home_dir().ok_or_else(|| {
                    KeyManagerError::InitializationError("Could not find home directory".into())
                })?
            }
            OperatingSystem::Unknown => std::env::current_dir().map_err(|_| {
                KeyManagerError::InitializationError("Could not determine current directory".into())
            })?,
        };

        Ok(base.join(".romer").join("keys"))
    }

    /// Gets the path where a permanent key of the specified scheme should be stored
    fn get_permanent_key_path(&self, scheme: SignatureScheme) -> PathBuf {
        self.permanent_dir.join(format!("{:?}.key", scheme))
    }

    /// Saves a permanent key to disk
    fn save_permanent_key(&self, scheme: SignatureScheme, key: &[u8]) -> KeyManagerResult<()> {
        let path = self.get_permanent_key_path(scheme);
        fs::write(&path, key).map_err(|e| KeyManagerError::IoError(e))
    }

    /// Saves session key data to disk
    fn save_session_key(&self, session_data: &SessionKeyData) -> KeyManagerResult<()> {
        // First, we need to convert the raw bytes into a PrivateKey type
        // This wraps our raw bytes in the proper type expected by the Scheme trait
        let session_private_key = PrivateKey::from(session_data.key_bytes.clone());

        // Now we can create a Bls12381 instance using the Scheme trait's from method
        // This converts the PrivateKey into a full BLS signer instance
        let session_key = <Bls12381 as Scheme>::from(session_private_key)
            .ok_or_else(|| KeyManagerError::InvalidKeyFormat("Invalid session key".into()))?;

        // Get the public key for the filename. The public_key() method returns PublicKey,
        // which we can convert to bytes using as_ref()
        let session_id = hex::encode(session_key.public_key().as_ref());
        let path = self.session_dir.join(format!("{}.json", session_id));

        let content = serde_json::to_string(session_data)
            .map_err(|e| KeyManagerError::SerializationError(e.to_string()))?;

        fs::write(&path, content).map_err(|e| KeyManagerError::IoError(e))
    }
}
