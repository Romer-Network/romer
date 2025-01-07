use crate::types::ValidatorLocation;
use crate::validation::{
    hardware_validator::{HardwareDetector, VirtualizationType},
    location_validator::LocationValidator,
};
use anyhow::{Context, Result};
use geo::Point;

pub struct ProofGeneratorBuilder {
    // Store validation results
    hardware_validation: Option<VirtualizationType>,
    location_validation: Option<ValidatorLocation>,
    // Store the validator instance for location checks
    location_validator: LocationValidator,
}

impl ProofGeneratorBuilder {
    pub fn new() -> Self {
        Self {
            hardware_validation: None,
            location_validation: None,
            location_validator: LocationValidator::new(),
        }
    }

    /// Validates that the node is running on physical hardware
    pub fn validate_hardware(mut self) -> Result<Self> {
        let virt_type = HardwareDetector::detect_virtualization()
            .context("Failed to perform hardware validation")?;

        match virt_type {
            VirtualizationType::Physical => {
                self.hardware_validation = Some(virt_type);
                Ok(self)
            }
            VirtualizationType::Virtual(platform) => Err(anyhow::anyhow!(
                "Node must run on physical hardware, detected virtualization platform: {}",
                platform
            )),
        }
    }

    /// Validates the claimed validator location by performing network measurements
    /// to verify physical presence at the specified coordinates.
    pub async fn validate_location(mut self, location: ValidatorLocation) -> Result<Self> {
        // Perform network-based location validation
        let validation_result = self
            .location_validator
            .validate_location(location.latitude(), location.longitude())
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .context("Failed to perform location validation measurements")?;

        // Check if validation succeeded based on confidence threshold
        if validation_result.is_valid {
            // Store the validated location
            self.location_validation = Some(location);
            Ok(self)
        } else {
            // Provide detailed error information if validation fails
            let error_details = validation_result.inconsistencies.join("\n  - ");
            Err(anyhow::anyhow!(
                "Location validation failed (confidence: {:.2})\nInconsistencies:\n  - {}",
                validation_result.confidence,
                error_details
            ))
        }
    }

    /// Checks if all required validations have been completed
    fn validations_complete(&self) -> bool {
        // Both hardware and location validations must be complete
        self.hardware_validation.is_some() && self.location_validation.is_some()
    }

    /// Builds the ProofGenerator if all validations pass
    pub fn build(self) -> Result<ProofGenerator> {
        if !self.validations_complete() {
            return Err(anyhow::anyhow!(
                "Cannot build ProofGenerator: not all validations completed"
            ));
        }

        Ok(ProofGenerator {
            hardware_validation: self.hardware_validation.unwrap(),
            location_validation: self.location_validation.unwrap(),
        })
    }
}

/// ProofGenerator represents a fully validated node that can generate
/// proofs of its validity for the network.
pub struct ProofGenerator {
    hardware_validation: VirtualizationType,
    location_validation: ValidatorLocation,
}

impl ProofGenerator {
    pub fn builder() -> ProofGeneratorBuilder {
        ProofGeneratorBuilder::new()
    }

    /// Returns the validated location of this node
    pub fn location(&self) -> &ValidatorLocation {
        &self.location_validation
    }
}