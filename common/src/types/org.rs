/// Represents the different types of organizations in the RØMER network
#[derive(Debug, Clone, PartialEq)]
pub enum OrganizationType {
    MarketMaker,
    StablecoinIssuer,
}

/// Represents an organization participating in the RØMER network
#[derive(Debug, Clone)]
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
    pub fn validate(&self) -> Result<(), String> {
        // Ensure ID is not empty
        if self.id.is_empty() {
            return Err("Organization ID cannot be empty".to_string());
        }

        // Ensure name is not empty
        if self.name.is_empty() {
            return Err("Organization name cannot be empty".to_string());
        }

        // Ensure sender_comp_id is not empty
        if self.sender_comp_id.is_empty() {
            return Err("Sender Comp ID cannot be empty".to_string());
        }

        // Ensure public key is present
        if self.public_key.is_empty() {
            return Err("Public key cannot be empty".to_string());
        }

        Ok(())
    }
}