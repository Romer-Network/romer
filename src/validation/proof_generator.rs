use anyhow::{Context, Result};
use geo::Point;
use crate::validation::{
    hardware_validator::{HardwareDetector, VirtualizationType},
    latency_validator::{LatencyValidator, LatencyConfig},
};
use std::net::IpAddr;

// Default reference point constants for Frankfurt IX
const DEFAULT_REF_LAT: f64 = 50.1109;
const DEFAULT_REF_LON: f64 = 8.6821;
const DEFAULT_REF_IP: &str = "80.81.192.3";

pub struct ProofGeneratorBuilder {
    // Validation state
    hardware_validation: Option<VirtualizationType>,
    location_validation: Option<Point<f64>>,
    
    // Reference point for validation
    reference_point: Point<f64>,
    reference_ip: IpAddr,
    
    // Latency validator instance
    latency_validator: LatencyValidator,
}

impl ProofGeneratorBuilder {
    pub fn new() -> Self {
        // Initialize with default Frankfurt reference point
        Self {
            hardware_validation: None,
            location_validation: None,
            reference_point: Point::new(DEFAULT_REF_LON, DEFAULT_REF_LAT),
            reference_ip: DEFAULT_REF_IP.parse().unwrap(),
            latency_validator: LatencyValidator::new(LatencyConfig::default()),
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
            VirtualizationType::Virtual(platform) => {
                Err(anyhow::anyhow!(
                    "Node must run on physical hardware, detected virtualization platform: {}",
                    platform
                ))
            }
        }
    }

    /// Validates the claimed location using latency measurements
    pub async fn validate_location(mut self, location: Point<f64>) -> Result<Self> {
        // Perform latency validation against reference point
        let validation_result = self.latency_validator
            .validate_latency(
                location,
                self.reference_point,
                self.reference_ip
            )
            .await
            .context("Failed to validate location using latency measurements")?;

        if validation_result.is_valid {
            self.location_validation = Some(location);
            Ok(self)
        } else {
            Err(anyhow::anyhow!(
                "Location validation failed: {}", 
                validation_result.details
            ))
        }
    }

    /// Optionally override the default reference point
    pub fn with_reference(mut self, point: Point<f64>, ip: IpAddr) -> Self {
        self.reference_point = point;
        self.reference_ip = ip;
        self
    }

    /// Checks if all required validations are complete
    fn validations_complete(&self) -> bool {
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

/// Represents a fully validated node that can generate proofs of its validity
pub struct ProofGenerator {
    hardware_validation: VirtualizationType,
    location_validation: Point<f64>,
}

impl ProofGenerator {
    pub fn builder() -> ProofGeneratorBuilder {
        ProofGeneratorBuilder::new()
    }

    /// Returns the validated location of this node
    pub fn location(&self) -> &Point<f64> {
        &self.location_validation
    }
}