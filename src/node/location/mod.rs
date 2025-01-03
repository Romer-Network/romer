mod measurements;
mod analysis;
mod types;

use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use geo::{Point, HaversineDistance};
use anyhow::{Result, Error};

pub use crate::node::location::types::*;
pub use crate::node::location::measurements::*;
pub use crate::node::location::analysis::*;

const SPEED_OF_LIGHT_KMS: f64 = 299792.458; // km/s
const FIBER_OVERHEAD: f64 = 1.4; // Typical fiber route overhead factor

pub struct LocationValidator {
    reference_points: Vec<ReferencePoint>,
    num_samples: usize,
    timeout_ms: u64,
    min_confidence_threshold: f64,
}

impl LocationValidator {
    pub fn new() -> Self {
        let reference_points = vec![
            ReferencePoint::new(
                "DE-CIX Frankfurt",
                "80.81.192.3".parse().unwrap(),
                50.1109,
                8.6821
            ),
            ReferencePoint::new(
                "LINX London",
                "195.66.224.1".parse().unwrap(),
                51.5074,
                -0.1278
            ),
            ReferencePoint::new(
                "AMS-IX Amsterdam",
                "80.249.208.1".parse().unwrap(),
                52.3676,
                4.9041
            ),
            ReferencePoint::new(
                "Cloudflare NYC",
                "104.18.0.0".parse().unwrap(),
                40.7128,
                -74.0060
            ),
        ];

        Self {
            reference_points,
            num_samples: 10,
            timeout_ms: 1000,
            min_confidence_threshold: 0.7,
        }
    }

    pub async fn validate_location(
        &self, 
        claimed_lat: f64, 
        claimed_lon: f64
    ) -> Result<LocationValidation> {
        let claimed_point = Point::new(claimed_lon, claimed_lat);
        let mut measurements = Vec::new();
        let mut path_analyses = Vec::new();
        
        let network_analyzer = NetworkAnalyzer::new();

        for reference in &self.reference_points {
            let measured_latency = self.measure_latency(reference.ip).await?;
            let path_analysis = network_analyzer.analyze_path(reference.ip).await?;
            
            measurements.push(LatencyMeasurement {
                reference: reference.clone(),
                measured_latency_ms: measured_latency,
                timestamp: Instant::now(),
                samples: vec![measured_latency],
            });

            path_analyses.push(path_analysis);
        }

        self.analyze_measurements_and_paths(claimed_point, measurements, path_analyses)
    }

    async fn measure_latency(&self, ip: IpAddr) -> Result<f64> {
        let mut samples = Vec::with_capacity(self.num_samples);
        
        for _ in 0..self.num_samples {
            let start = Instant::now();
            
            match timeout(
                Duration::from_millis(self.timeout_ms),
                TcpStream::connect((ip, 80))
            ).await {
                Ok(Ok(_)) => {
                    samples.push(start.elapsed().as_secs_f64() * 1000.0);
                }
                Ok(Err(e)) => {
                    tracing::warn!("Connection failed to {}: {}", ip, e);
                    continue;
                }
                Err(_) => {
                    tracing::warn!("Connection timeout to {}", ip);
                    continue;
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        if samples.is_empty() {
            return Err(Error::msg("No successful latency measurements"));
        }

        Ok(self.calculate_median_latency(samples))
    }

    fn calculate_median_latency(&self, mut samples: Vec<f64>) -> f64 {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = samples.len() / 2;
        if samples.len() % 2 == 0 {
            (samples[mid - 1] + samples[mid]) / 2.0
        } else {
            samples[mid]
        }
    }

    fn analyze_measurements_and_paths(
        &self,
        claimed_location: Point<f64>,
        measurements: Vec<LatencyMeasurement>,
        path_analyses: Vec<NetworkPath>,
    ) -> Result<LocationValidation> {
        let mut confidence = 1.0;
        let mut inconsistencies = Vec::new();

        for (measurement, path) in measurements.iter().zip(path_analyses.iter()) {
            let reference = &measurement.reference;
            let min_latency = reference.calculate_min_latency(claimed_location);
            
            // Physics-based checks
            if measurement.measured_latency_ms < min_latency {
                confidence *= 0.1;
                inconsistencies.push(format!(
                    "{}: Measured latency {}ms below physical minimum {}ms",
                    reference.name,
                    measurement.measured_latency_ms,
                    min_latency
                ));
            }

            // Path analysis checks
            if !path.suspicious_patterns.is_empty() {
                confidence *= 0.5;
                inconsistencies.extend(path.suspicious_patterns.clone());
            }

            // Latency consistency checks
            let expected_hops = (min_latency / 20.0).ceil() as usize;
            if path.path_length > expected_hops * 2 {
                confidence *= 0.8;
                inconsistencies.push(format!(
                    "Path to {} has too many hops: {} (expected ~{})",
                    reference.name,
                    path.path_length,
                    expected_hops
                ));
            }
        }

        Ok(LocationValidation {
            confidence,
            inconsistencies,
            measurements,
            path_analyses: Some(path_analyses),
            is_valid: confidence >= self.min_confidence_threshold,
        })
    }
}