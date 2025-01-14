use commonware_storage::journal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::storage::journal::Partition;
use crate::storage::journal::Section;

use crate::{
    storage::journal::{JournalEntry, RomerJournal},
    types::keymanager::KeyManagerError,
};

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum OrganizationError {
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Invalid sender comp ID: {0}")]
    InvalidSenderCompId(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid organization type: {0}")]
    InvalidType(String),

    #[error("Organization not found: {0}")]
    NotFound(String),

    #[error("Organization already exists: {0}")]
    AlreadyExists(String),
}

pub type OrganizationResult<T> = Result<T, OrganizationError>;

// Now for the registration-specific errors that compose different error types
#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("Organization error: {0}")]
    Organization(#[from] OrganizationError),

    #[error("Key management error: {0}")]
    KeyManager(#[from] KeyManagerError),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Runtime error: {0}")]
    Runtime(#[from] std::io::Error),
}

// A type alias for Results involving RegistrationError
pub type RegistrationResult<T> = Result<T, RegistrationError>;

/// Represents the different types of organizations in the RØMER network
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrganizationType {
    MarketMaker,
    BrokerDealer,
    Bank,
    AssetManager,
    InfraProvider,
    ServiceProvider,
    PrimeBroker,
    Custodian,
}

pub struct OrganizationManager {
    organization: Organization,
    journal: RomerJournal,
}
/// Represents an organization participating in the RØMER network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Unique identifier for the organization
    pub id: String,

    /// Organization's registered name
    pub name: String,

    /// Type of organization and its role in the network
    pub org_type: OrganizationType,

    /// FIX protocol sender comp ID for message routing
    pub sender_comp_id: String,

    /// BLS public key used for cryptographic operations
    pub public_key: Vec<u8>,

    /// Timestamp of registration (Unix timestamp in seconds)
    pub registered_at: u64,
}

impl Organization {
    /// Creates a new organization with the current timestamp
    pub fn new(
        id: String,
        name: String,
        org_type: OrganizationType,
        sender_comp_id: String,
        public_key: Vec<u8>,
    ) -> Self {
        // Get current Unix timestamp in seconds
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            name,
            org_type,
            sender_comp_id,
            public_key,
            registered_at: now,
        }
    }

    /// Validates the organization's data
    /// Validates the organization's data, now returning OrganizationResult
    pub fn validate(&self) -> OrganizationResult<()> {
        // Ensure ID is not empty and has valid format
        if self.id.is_empty() {
            return Err(OrganizationError::InvalidIdentifier(
                "Organization ID cannot be empty".into(),
            ));
        }

        // Ensure name meets requirements
        if self.name.is_empty() {
            return Err(OrganizationError::InvalidName(
                "Organization name cannot be empty".into(),
            ));
        }

        // More detailed name validation could go here
        if self.name.len() < 3 {
            return Err(OrganizationError::InvalidName(
                "Organization name must be at least 3 characters".into(),
            ));
        }

        // Validate sender_comp_id format and constraints
        if self.sender_comp_id.is_empty() {
            return Err(OrganizationError::InvalidSenderCompId(
                "Sender Comp ID cannot be empty".into(),
            ));
        }

        // Additional sender_comp_id validation could go here
        if !self
            .sender_comp_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(OrganizationError::InvalidSenderCompId(
                "Sender Comp ID can only contain alphanumeric characters and underscores".into(),
            ));
        }

        // Ensure public key is present and valid
        if self.public_key.is_empty() {
            return Err(OrganizationError::InvalidPublicKey(
                "Public key cannot be empty".into(),
            ));
        }

        // Additional public key validation could go here
        if self.public_key.len() != 48 {
            // Assuming BLS12-381
            return Err(OrganizationError::InvalidPublicKey(
                "Invalid public key length".into(),
            ));
        }

        Ok(())
    }

    pub async fn write_to_journal(&self) -> RegistrationResult<()> {
        self.validate()?;

        let mut journal = RomerJournal::new(Partition::SYSTEM, Section::ORGANIZATION)
            .await
            .map_err(|e| RegistrationError::Storage(e.to_string()))?;

        let entry = JournalEntry::OrganizationRegistered(self.clone());
        let bytes = serde_json::to_vec(&entry).expect("Issue with the Bytes");

        journal
            .journal
            .append(1, bytes.into())
            .await
            .map_err(|e| RegistrationError::Storage(e.to_string()))?;

        journal
            .journal
            .sync(1)
            .await
            .map_err(|e| RegistrationError::Storage(e.to_string()))?;

        Ok(())
    }

    pub async fn get_all_organizations(&self) -> Result<Vec<Organization>, String> {
        let mut organizations = Vec::new();

        Ok(organizations)
    }
}
