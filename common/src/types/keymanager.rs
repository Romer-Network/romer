use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

/// Represents the supported signature schemes in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureScheme {
    Ed25519,
    Bls12381,
}

/// Represents a session key along with its metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionKeyData {
    /// The raw bytes of the session key
    pub key_bytes: Vec<u8>,
    /// When the session key was created
    pub created_at: DateTime<Utc>,
    /// When the session key expires
    pub expires_at: DateTime<Utc>,
    /// The public key of the permanent key that signed this session
    pub parent_public_key: Vec<u8>,
    /// The signature from the parent key validating this session key
    pub parent_signature: Vec<u8>,
    /// Purpose of this session key (e.g., "FIX")
    pub purpose: String,
    /// The namespace this session key operates within.
    /// For FIX sessions this would be the SenderCompID,
    /// for other use cases it could be different identifiers.
    pub namespace: String,
}

/// Custom error types for key management operations
#[derive(Error, Debug)]
pub enum KeyManagerError {
    #[error("Failed to initialize key storage: {0}")]
    InitializationError(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Session key expired")]
    SessionExpired,

    #[error("Invalid session signature")]
    InvalidSessionSignature,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage directory error: {0}")]
    StorageError(String),
}

/// Result type alias for key management operations
pub type KeyManagerResult<T> = Result<T, KeyManagerError>;