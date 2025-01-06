mod analysis;
mod measurements;
mod types;

use anyhow::{Error, Result};
use geo::{HaversineDistance, Point};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{error, info, warn};

pub use crate::node::location_validator::analysis::*;
pub use crate::node::location_validator::measurements::*;
pub use crate::node::location_validator::types::*;

const SPEED_OF_LIGHT_KMS: f64 = 299792.458; // km/s
const FIBER_OVERHEAD: f64 = 1.4; // Typical fiber route overhead factor

pub struct LocationValidator {
    reference_points: Vec<ReferencePoint>,
    network_measurement: NetworkMeasurement,
    network_analyzer: NetworkAnalyzer,
}

impl LocationValidator {
    pub fn new() -> Self {
        // Default reference points matching the existing implementation
        let reference_points = vec![
            ReferencePoint::new(
                "DE-CIX Frankfurt",
                "80.81.192.3".parse().unwrap(),
                50.1109,
                8.6821,
            ),
            ReferencePoint::new(
                "Trollip",
                "27.33.41.4".parse().unwrap(),
                -28.0167,
                153.4000,
            ),
        ];

        Self {
            reference_points,
            network_measurement: NetworkMeasurement::new(MeasurementConfig::default()),
            network_analyzer: NetworkAnalyzer::new(),
        }
    }

    pub async fn validate_location(
        &self,
        claimed_lat: f64,
        claimed_lon: f64,
    ) -> Result<LocationValidation, String> {
        // Step 1: Measure latency to reference points
        let mut latency_measurements = Vec::new();

        let claimed_point = Point::new(claimed_lon, claimed_lat);

        for reference in &self.reference_points {
            // Measure latency
            let latency_samples = self
                .network_measurement
                .measure_latency(reference.ip)
                .await
                .map_err(|e| format!("Latency measurement failed for {}: {}", reference.name, e))?;

            // Create latency measurement
            let mean_latency = latency_samples.iter().sum::<f64>() / latency_samples.len() as f64;

            latency_measurements.push(LatencyMeasurement {
                reference: reference.clone(),
                measured_latency_ms: mean_latency,
                timestamp: Instant::now(),
                samples: latency_samples,
            });
        }

        // Step 2: Analyze measurements using NetworkAnalyzer
        // Note: Removed path-related analysis
        let location_validation = self
            .network_analyzer
            .analyze_measurements(claimed_point, &latency_measurements)
            .map_err(|e| format!("Location analysis failed: {}", e))?;

        Ok(location_validation)
    }
}
