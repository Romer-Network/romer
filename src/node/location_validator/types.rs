use std::net::IpAddr;
use std::time::Instant;
use geo::{Point, HaversineDistance};

/// Represents a known network reference point used for location validation.
/// These are typically major internet exchanges or well-known network nodes
/// with stable infrastructure and known physical locations.
#[derive(Debug, Clone)]
pub struct ReferencePoint {
    /// Human-readable name of the reference point (e.g., "DE-CIX Frankfurt")
    pub name: String,
    
    /// IP address of the reference point for network measurements
    pub ip: IpAddr,
    
    /// Physical location of the reference point as a geographic coordinate
    pub location: Point<f64>,
    
    /// Theoretical minimum latency based on speed of light calculations
    pub min_latency_ms: f64,
}

impl ReferencePoint {
    pub fn new(name: &str, ip: IpAddr, lat: f64, lon: f64) -> Self {
        let location = Point::new(lon, lat);
        Self {
            name: name.to_string(),
            ip,
            location,
            min_latency_ms: 0.0, // Will be calculated based on claimed location
        }
    }

    /// Calculates the theoretical minimum latency to this reference point from a given location
    /// based on the speed of light through fiber optic cables.
    pub fn calculate_min_latency(&self, claimed_location: Point<f64>) -> f64 {
        const SPEED_OF_LIGHT_KMS: f64 = 299792.458; // Speed of light in km/s
        const FIBER_OVERHEAD: f64 = 1.4; // Typical fiber route overhead factor
        
        // Calculate great circle distance between points
        let distance_km = self.location.haversine_distance(&claimed_location);
        
        // Calculate round trip time in milliseconds, accounting for:
        // 1. Round trip (multiply by 2)
        // 2. Fiber overhead (typical cable routes are longer than great circle)
        // 3. Speed of light in fiber
        (distance_km * FIBER_OVERHEAD * 2.0 / SPEED_OF_LIGHT_KMS) * 1000.0
    }
}

/// Records a latency measurement to a reference point
#[derive(Debug, Clone)]
pub struct LatencyMeasurement {
    /// The reference point being measured
    pub reference: ReferencePoint,
    
    /// Measured round trip time in milliseconds
    pub measured_latency_ms: f64,
    
    /// When this measurement was taken
    pub timestamp: Instant,
    
    /// Collection of individual latency samples
    pub samples: Vec<f64>,
}

/// Contains the complete results of a location validation attempt
#[derive(Debug)]
pub struct LocationValidation {
    /// Confidence score from 0.0 to 1.0 indicating likelihood that
    /// the claimed location is accurate
    pub confidence: f64,
    
    /// List of specific issues found during validation
    pub inconsistencies: Vec<String>,
    
    /// Collection of latency measurements to reference points
    pub measurements: Vec<LatencyMeasurement>,
    
    /// Whether the location claim meets our minimum confidence threshold
    pub is_valid: bool,
}

/// Represents the possible results of a verification attempt
#[derive(Debug)]
pub enum VerificationResult {
    /// Location verified successfully
    Verified {
        confidence: f64,
        validations: LocationValidation,
    },
    
    /// Verification failed with specific reasons
    Failed {
        reasons: Vec<String>,
        validations: LocationValidation,
    },
    
    /// Verification encountered an error
    Error(String),
}

/// Configuration options for the location validator
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Number of latency samples to collect per reference point
    pub samples_per_reference: usize,
    
    /// Timeout for individual network measurements in milliseconds
    pub measurement_timeout_ms: u64,
    
    /// Minimum confidence score required to consider a location verified
    pub min_confidence_threshold: f64,
    
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            samples_per_reference: 10,
            measurement_timeout_ms: 1000,
            min_confidence_threshold: 0.7,
        }
    }
}