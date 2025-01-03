use anyhow::{Error, Result};
use geo::{HaversineDistance, Point};
use std::{net::IpAddr, time::Instant};
use tracing::{debug, warn};

use crate::node::location_validator::types::{
    LatencyMeasurement, LocationValidation, NetworkPath, PathHop, ReferencePoint,
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

    fn check_latency_ratios(
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

    // Add this method
    fn check_shared_paths(&self, paths: &[NetworkPath]) -> Option<Vec<String>> {
        let mut issues = Vec::new();

        // Look for suspicious patterns in shared path segments
        for i in 0..paths.len() {
            for j in i + 1..paths.len() {
                let shared_hops = self.find_shared_hops(&paths[i], &paths[j]);

                if shared_hops > 3 {
                    issues.push(format!(
                        "Suspicious number of shared hops ({}) between different paths",
                        shared_hops
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

    fn find_shared_hops(&self, path1: &NetworkPath, path2: &NetworkPath) -> usize {
        let mut shared = 0;

        for hop1 in &path1.hops {
            for hop2 in &path2.hops {
                if hop1.responded && hop2.responded && hop1.ip == hop2.ip {
                    shared += 1;
                }
            }
        }

        shared
    }

    pub async fn analyze_path(&self, target: IpAddr) -> Result<NetworkPath> {
        // First, we need to perform the actual traceroute measurement
        let hops = self.measure_path(target).await?;

        // Calculate the timing characteristics for the path
        let (consistency_score, avg_latency) = self.calculate_path_metrics(&hops);

        // Create initial NetworkPath structure
        let mut path = NetworkPath {
            hops: hops.clone(), // Clone the hops vector
            suspicious_patterns: Vec::new(),
            average_inter_hop_latency: avg_latency,
            latency_consistency_score: consistency_score,
            path_length: hops.len(),
        };
        // Use our existing analysis functions to detect suspicious patterns
        let analysis = self.analyze_path_characteristics(&path);
        path.suspicious_patterns = analysis.issues;

        Ok(path)
    }

    // Add this helper function to perform the actual path measurement
    async fn measure_path(&self, target: IpAddr) -> Result<Vec<PathHop>> {
        use tokio::process::Command;

        // Run traceroute command
        let output = Command::new("traceroute")
            .arg("-n") // Numeric output only
            .arg("-q")
            .arg("3") // 3 probes per hop
            .arg("-w")
            .arg("2") // 2 second timeout
            .arg(target.to_string())
            .output()
            .await?;

        let output_str = String::from_utf8(output.stdout)?;

        // Parse the traceroute output into PathHop structures
        let mut hops = Vec::new();

        for line in output_str.lines().skip(1) {
            // Skip header line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            // Handle non-responding hops
            let (ip, rtt, responded) = if parts[1] == "*" {
                (IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0.0, false)
            } else {
                let ip: IpAddr = parts[1].parse()?;
                let rtt: f64 = parts[2].trim_end_matches("ms").parse()?;
                (ip, rtt, true)
            };

            hops.push(PathHop { ip, rtt, responded });
        }

        Ok(hops)
    }

    // Add this helper function to calculate path metrics
    fn calculate_path_metrics(&self, hops: &[PathHop]) -> (f64, f64) {
        let mut latencies = Vec::new();
        let mut prev_rtt = 0.0;

        for hop in hops.iter().filter(|h| h.responded) {
            if prev_rtt > 0.0 {
                latencies.push(hop.rtt - prev_rtt);
            }
            prev_rtt = hop.rtt;
        }

        if latencies.is_empty() {
            return (0.0, 0.0);
        }

        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;

        // Calculate consistency score using coefficient of variation
        let variance = latencies
            .iter()
            .map(|&lat| (lat - avg_latency).powi(2))
            .sum::<f64>()
            / latencies.len() as f64;

        let std_dev = variance.sqrt();
        let consistency_score = 1.0 / (1.0 + (std_dev / avg_latency));

        (consistency_score, avg_latency)
    }

    /// Performs comprehensive analysis of network measurements to validate
    /// a location claim. Combines latency analysis, path characteristics,
    /// and physical constraints to generate a confidence score.
    pub fn analyze_measurements(
        &self,
        claimed_location: Point<f64>,
        measurements: &[LatencyMeasurement],
        paths: &[NetworkPath],
    ) -> Result<LocationValidation> {
        debug!("Starting comprehensive location analysis");

        let mut confidence = 1.0;
        let mut inconsistencies = Vec::new();

        // Analyze each reference point measurement
        for (measurement, path) in measurements.iter().zip(paths.iter()) {
            let analysis_result =
                self.analyze_single_reference(claimed_location, measurement, path)?;

            confidence *= analysis_result.confidence_factor;
            inconsistencies.extend(analysis_result.issues);
        }

        // Perform cross-reference analysis to detect coordination
        if let Some(cross_issues) = self.analyze_cross_references(
            claimed_location, // Pass claimed_location here
            measurements,
            paths,
        ) {
            confidence *= 0.5; // Significant penalty for cross-reference issues
            inconsistencies.extend(cross_issues);
        }

        Ok(LocationValidation {
            confidence,
            inconsistencies,
            measurements: measurements.to_vec(),
            path_analyses: Some(paths.to_vec()),
            is_valid: confidence >= 0.7, // Minimum threshold for validation
        })
    }

    /// Analyzes measurements from a single reference point against physical
    /// and network constraints to detect anomalies.
    fn analyze_single_reference(
        &self,
        claimed_location: Point<f64>,
        measurement: &LatencyMeasurement,
        path: &NetworkPath,
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

        // Analyze path characteristics
        let path_analysis = self.analyze_path_characteristics(path);
        confidence *= path_analysis.confidence_factor;
        issues.extend(path_analysis.issues);

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

    /// Analyzes network path characteristics to detect tunneling,
    /// proxying, or other suspicious routing patterns.
    fn analyze_path_characteristics(&self, path: &NetworkPath) -> PathAnalysis {
        let mut confidence = 1.0;
        let mut issues = Vec::new();

        // Check for suspiciously consistent latencies
        if path.latency_consistency_score > self.suspicious_consistency_threshold {
            confidence *= 0.7;
            issues.push("Suspiciously consistent inter-hop latencies".to_string());
        }

        // Analyze hop latency patterns
        self.analyze_hop_latencies(path, &mut confidence, &mut issues);

        // Check for suspicious gaps
        self.analyze_path_gaps(path, &mut confidence, &mut issues);

        PathAnalysis {
            confidence_factor: confidence,
            issues,
        }
    }

    /// Analyzes latency patterns between consecutive hops to detect
    /// impossible or unlikely patterns that might indicate tunneling.
    fn analyze_hop_latencies(
        &self,
        path: &NetworkPath,
        confidence: &mut f64,
        issues: &mut Vec<String>,
    ) {
        let mut prev_hop: Option<&PathHop> = None;

        for hop in path.hops.iter().filter(|h| h.responded) {
            if let Some(prev) = prev_hop {
                let hop_latency = hop.rtt - prev.rtt;

                // Check for physically impossible decreases
                if hop_latency < 0.0 {
                    *confidence *= 0.3;
                    issues.push(format!(
                        "Physically impossible latency decrease: {}ms",
                        hop_latency
                    ));
                }

                // Check for suspiciously small inter-hop latencies
                if hop_latency < self.min_hop_latency {
                    *confidence *= 0.7;
                    issues.push(format!(
                        "Suspiciously small inter-hop latency: {}ms",
                        hop_latency
                    ));
                }
            }
            prev_hop = Some(hop);
        }
    }

    /// Analyzes gaps in the path where hops don't respond, which
    /// might indicate tunneling or network manipulation.
    fn analyze_path_gaps(
        &self,
        path: &NetworkPath,
        confidence: &mut f64,
        issues: &mut Vec<String>,
    ) {
        let mut current_gap = 0;

        for hop in &path.hops {
            if !hop.responded {
                current_gap += 1;
            } else if current_gap >= self.suspicious_gap_size {
                *confidence *= 0.8;
                issues.push(format!(
                    "Suspicious gap of {} non-responding hops",
                    current_gap
                ));
                current_gap = 0;
            } else {
                current_gap = 0;
            }
        }
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
        paths: &[NetworkPath],
    ) -> Option<Vec<String>> {
        let mut issues = Vec::new();

        // Check for consistent latency ratios across references
        if let Some(ratio_issues) = self.check_latency_ratios(measurements, claimed_location) {
            issues.extend(ratio_issues);
        }

        // Check for shared path segments indicating tunneling
        if let Some(path_issues) = self.check_shared_paths(paths) {
            issues.extend(path_issues);
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues)
        }
    }
}

/// Holds the results of analyzing a single reference point
struct ReferenceAnalysis {
    confidence_factor: f64,
    issues: Vec<String>,
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
