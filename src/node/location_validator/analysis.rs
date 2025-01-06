use anyhow::{Error, Result};
use geo::{HaversineDistance, Point};
use std::{net::IpAddr, time::Instant};
use tracing::{debug, warn};

use crate::node::location_validator::types::{
    LatencyMeasurement, LocationValidation, ReferencePoint,
};

/// The NetworkAnalyzer performs sophisticated analysis of network measurements
/// to validate geographic location claims. It uses principles of physics and
/// network behavior to detect inconsistencies and potential deception.
pub struct NetworkAnalyzer {
    /// Minimum physically possible time between network hops in milliseconds,
    /// based on speed of light in fiber and minimum processing time
    min_hop_latency: f64,

    /// Maximum ratio of measured latency to theoretical minimum before
    /// considering it suspicious
    max_latency_ratio: f64,

    /// Number of consecutive non-responding hops that indicates potential tunneling
    suspicious_gap_size: usize,

    /// Threshold for latency consistency score above which the path
    /// might indicate tunneling (real paths have more variance)
    suspicious_consistency_threshold: f64,
}

impl NetworkAnalyzer {
    pub fn new() -> Self {
        Self {
            min_hop_latency: 0.1,   // 100 microseconds minimum between hops
            max_latency_ratio: 2.5, // Max 2.5x theoretical minimum latency
            suspicious_gap_size: 3,
            suspicious_consistency_threshold: 0.95,
        }
    }

    pub fn check_latency_ratios(
        &self,
        measurements: &[LatencyMeasurement],
        claimed_location: Point<f64>,
    ) -> Option<Vec<String>> {
        let mut issues = Vec::new();

        // For each pair of measurements
        for i in 0..measurements.len() {
            for j in i + 1..measurements.len() {
                let ratio =
                    measurements[i].measured_latency_ms / measurements[j].measured_latency_ms;

                // Calculate expected ratio based on distances
                let expected_ratio = measurements[i]
                    .reference
                    .calculate_min_latency(claimed_location)
                    / measurements[j]
                        .reference
                        .calculate_min_latency(claimed_location);

                // If measured ratio deviates significantly from expected
                if (ratio - expected_ratio).abs() > 0.5 {
                    issues.push(format!(
                        "Suspicious latency ratio between {} and {}: expected {:.2}, got {:.2}",
                        measurements[i].reference.name,
                        measurements[j].reference.name,
                        expected_ratio,
                        ratio
                    ));
                }
            }
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }

    /// Performs comprehensive analysis of network measurements to validate
    /// a location claim. Combines latency analysis
    /// and physical constraints to generate a confidence score.
    pub fn analyze_measurements(
        &self,
        claimed_location: Point<f64>,
        measurements: &[LatencyMeasurement],
    ) -> Result<LocationValidation> {
        debug!("Starting comprehensive location analysis");

        let mut confidence = 1.0;
        let mut inconsistencies = Vec::new();

        // Analyze each reference point measurement
        for measurement in measurements {
            let analysis_result = self.analyze_single_reference(claimed_location, measurement)?;

            confidence *= analysis_result.confidence_factor;
            inconsistencies.extend(analysis_result.issues);
        }

        // Perform cross-reference analysis to detect coordination
        if let Some(cross_issues) = self.analyze_cross_references(
            claimed_location, // Pass claimed_location here
            measurements,
        ) {
            confidence *= 0.5; // Significant penalty for cross-reference issues
            inconsistencies.extend(cross_issues);
        }

        Ok(LocationValidation {
            confidence,
            inconsistencies,
            measurements: measurements.to_vec(),
            is_valid: confidence >= 0.7, // Minimum threshold for validation
        })
    }

    /// Analyzes measurements from a single reference point against physical
    /// and network constraints to detect anomalies.
    pub fn analyze_single_reference(
        &self,
        claimed_location: Point<f64>,
        measurement: &LatencyMeasurement,
    ) -> Result<ReferenceAnalysis> {
        let mut confidence = 1.0;
        let mut issues = Vec::new();

        // Check physical constraints
        let min_latency =
            self.calculate_theoretical_minimum(claimed_location, &measurement.reference);

        if measurement.measured_latency_ms < min_latency {
            confidence *= 0.1; // Severe penalty for breaking physics
            issues.push(format!(
                "{}: Measured latency {}ms violates physical minimum {}ms",
                measurement.reference.name, measurement.measured_latency_ms, min_latency
            ));
        }

        // Check temporal consistency
        if let Some(temporal_issues) = self.check_temporal_consistency(measurement) {
            confidence *= 0.8;
            issues.extend(temporal_issues);
        }

        Ok(ReferenceAnalysis {
            confidence_factor: confidence,
            issues,
        })
    }

    /// Calculates the theoretical minimum latency between two points based on
    /// the speed of light in fiber optic cables and necessary network overhead.
    fn calculate_theoretical_minimum(
        &self,
        point_a: Point<f64>,
        reference: &ReferencePoint,
    ) -> f64 {
        const SPEED_OF_LIGHT_KMS: f64 = 299792.458;
        const FIBER_OVERHEAD: f64 = 1.4;
        const PROCESSING_OVERHEAD_MS: f64 = 0.2;

        let distance_km = point_a.haversine_distance(&reference.location);

        // Calculate round trip time including:
        // 1. Fiber optic travel time (speed of light / fiber index)
        // 2. Path overhead (cables don't follow great circles)
        // 3. Minimum processing time at each end
        let light_time = (distance_km * FIBER_OVERHEAD * 2.0 / SPEED_OF_LIGHT_KMS) * 1000.0;
        light_time + PROCESSING_OVERHEAD_MS
    }

    /// Analyzes temporal aspects of measurements to detect inconsistencies
    /// that might indicate replay or manipulation.
    fn check_temporal_consistency(&self, measurement: &LatencyMeasurement) -> Option<Vec<String>> {
        let mut issues = Vec::new();
        let elapsed = Instant::now().duration_since(measurement.timestamp);

        // Check for suspiciously old measurements
        if elapsed.as_secs() > 300 {
            issues.push(format!(
                "Measurement age ({} seconds) exceeds freshness threshold",
                elapsed.as_secs()
            ));
        }

        // Check for variance in samples
        if measurement.samples.len() >= 2 {
            let variance = calculate_sample_variance(&measurement.samples);
            if variance < 0.001 {
                issues.push("Suspiciously low variance in latency samples".to_string());
            }
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }

    /// Analyzes relationships between different reference point measurements
    /// to detect coordinated deception attempts.
    fn analyze_cross_references(
        &self,
        claimed_location: Point<f64>,
        measurements: &[LatencyMeasurement],
    ) -> Option<Vec<String>> {
        let mut issues = Vec::new();

        // Check for consistent latency ratios across references
        if let Some(ratio_issues) = self.check_latency_ratios(measurements, claimed_location) {
            issues.extend(ratio_issues);
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }
}

/// Holds the results of analyzing a single reference point
pub struct ReferenceAnalysis {
    pub confidence_factor: f64,
    pub issues: Vec<String>,
}

/// Holds the results of analyzing a network path
struct PathAnalysis {
    confidence_factor: f64,
    issues: Vec<String>,
}

/// Calculates the variance of a sample set
fn calculate_sample_variance(samples: &[f64]) -> f64 {
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / samples.len() as f64;
    variance
}
