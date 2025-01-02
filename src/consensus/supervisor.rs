use commonware_cryptography::{PublicKey};
use bytes::Bytes;
use commonware_consensus::Supervisor;
use tracing::info;

/// BlockchainSupervisor manages validator participation and leader selection
/// for the consensus process. It ensures proper coordination of validators
/// across different views of consensus.
#[derive(Clone)]
pub struct BlockchainSupervisor {
    // Store the validator's own public key for leader selection
    pub validator_key: PublicKey,
    // Track the current set of active validators
    active_validators: Vec<PublicKey>,
}

impl BlockchainSupervisor {
    pub fn new(validator_key: PublicKey) -> Self {
        Self {
            validator_key: validator_key.clone(),
            active_validators: vec![validator_key], // Start with self as only validator
        }
    }

    /// Updates the set of active validators
    pub fn update_validators(&mut self, validators: Vec<PublicKey>) {
        self.active_validators = validators;
        info!(
            "Updated active validator set. Count: {}",
            self.active_validators.len()
        );
    }

    /// Internal helper to determine if a validator is active
    fn is_active_validator(&self, candidate: &PublicKey) -> bool {
        self.active_validators.contains(candidate)
    }
}

impl Supervisor for BlockchainSupervisor {
    type Index = u64;  // View number type
    type Seed = ();    // No additional randomness needed yet

    fn leader(&self, _index: Self::Index, _seed: Self::Seed) -> Option<PublicKey> {
        // For now, always return self as leader
        // In the future, implement proper leader rotation based on view number
        Some(self.validator_key.clone())
    }

    fn participants(&self, _index: Self::Index) -> Option<&Vec<PublicKey>> {
        // Return the current set of active validators
        Some(&self.active_validators)
    }

    fn is_participant(&self, _index: Self::Index, candidate: &PublicKey) -> Option<u32> {
        // Check if the candidate is an active validator
        if self.is_active_validator(candidate) {
            // Return 0 as the validator index for now
            // In the future, implement proper validator indexing
            Some(0)
        } else {
            None
        }
    }

    async fn report(&self, _activity: u8, _proof: Bytes) {
        // Handle validator activity reports
        // This will be important for implementing validator scoring
        // and performance tracking in the future
    }
}