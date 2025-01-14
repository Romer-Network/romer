use thiserror::Error;
use std::io;

use crate::types::{keymanager::KeyManagerError, org::RegistrationError};

/// Core error types for the Rømer system. These serve as the foundation
/// for error handling across all components.
#[derive(Error, Debug)]
pub enum RomerError {
    /// Errors related to FIX protocol operations
    #[error("FIX protocol error: {0}")]
    Fix(#[from] FixError),

    /// Errors related to client operations
    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    /// Errors related to infrastructure/system operations
    #[error("System error: {0}")]
    System(#[from] SystemError),

    #[error("Key management error: {0}")]
    KeyManager(#[from] KeyManagerError),

    /// Catch-all for errors that don't fit other categories
    #[error("Other error: {0}")]
    Other(String),
}

/// Errors specific to FIX protocol operations
#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Sequence number error: {0}")]
    SequenceError(String),
}

/// Errors related to client operations
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// System-level infrastructure errors
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Resource error: {0}")]
    Resource(String),
}

/// Result type alias for Rømer operations
pub type RomerResult<T> = Result<T, RomerError>;

impl From<io::Error> for RomerError {
    fn from(error: io::Error) -> Self {
        // We'll convert io::Error to RomerError via ClientError
        RomerError::Client(ClientError::Io(error))
    }
}