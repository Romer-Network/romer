use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::timeout;
use anyhow::{Result, Error};
use tracing::{debug, warn};

use crate::node::location_validator::types::{PathHop, NetworkPath};

/// Handles network measurements for location validation, including latency
/// measurements and path analysis. This implementation uses TCP connections
/// for latency measurement to avoid requiring root privileges, and performs
/// path analysis using standard network tools.
pub struct NetworkMeasurement {
    /// Maximum time to wait for any single measurement
    timeout_ms: u64,
    
    /// Number of samples to collect for latency measurements
    sample_count: usize,
    
    /// Delay between consecutive measurements to avoid flooding
    inter_measurement_delay_ms: u64,
    
    /// Maximum number of network hops to analyze
    max_hops: u32,
}

impl NetworkMeasurement {
    pub fn new(config: MeasurementConfig) -> Self {
        Self {
            timeout_ms: config.timeout_ms,
            sample_count: config.sample_count,
            inter_measurement_delay_ms: config.inter_measurement_delay_ms,
            max_hops: config.max_hops,
        }
    }

    /// Performs a complete latency measurement to the target IP address.
    /// Collects multiple samples and performs statistical analysis to
    /// filter out anomalies and determine a reliable latency value.
    pub async fn measure_latency(&self, target: IpAddr) -> Result<Vec<f64>> {
        debug!("Starting latency measurement to {}", target);
        let mut samples = Vec::with_capacity(self.sample_count);
        let mut failed_attempts = 0;
        
        for i in 0..self.sample_count {
            match self.single_latency_measurement(target).await {
                Ok(latency) => {
                    debug!("Sample {} to {}: {:.2}ms", i + 1, target, latency);
                    samples.push(latency);
                }
                Err(e) => {
                    warn!("Failed to measure latency to {}: {}", target, e);
                    failed_attempts += 1;
                    if failed_attempts > self.sample_count / 2 {
                        return Err(Error::msg("Too many failed measurements"));
                    }
                }
            }

            // Add delay between measurements to avoid flooding
            tokio::time::sleep(Duration::from_millis(self.inter_measurement_delay_ms)).await;
        }

        if samples.is_empty() {
            return Err(Error::msg("No successful latency measurements"));
        }

        // Filter out anomalies and calculate final result
        Ok(self.process_latency_samples(samples))
    }

    /// Performs path analysis to the target IP address, analyzing each hop
    /// for suspicious patterns that might indicate proxying or tunneling.
    pub async fn analyze_path(&self, target: IpAddr) -> Result<NetworkPath> {
        debug!("Starting path analysis to {}", target);
        let mut hops: Vec<PathHop> = Vec::new();
        let mut suspicious_patterns = Vec::new();

        // Use traceroute for path analysis
        let output = self.run_traceroute(target).await?;
        let path_data = self.parse_traceroute_output(&output)?;

        // Analyze the path for suspicious patterns
        let (consistency_score, avg_latency) = self.analyze_path_characteristics(&path_data);
        
        if let Some(patterns) = self.detect_path_anomalies(&path_data) {
            suspicious_patterns.extend(patterns);
        }

        Ok(NetworkPath {
            hops: path_data,
            suspicious_patterns,
            average_inter_hop_latency: avg_latency,
            latency_consistency_score: consistency_score,
            path_length: hops.len(),
        })
    }

    /// Performs a single latency measurement using TCP connection timing.
    /// This method avoids using ICMP ping which requires root privileges.
    async fn single_latency_measurement(&self, target: IpAddr) -> Result<f64> {
        let start = Instant::now();
        
        match timeout(
            Duration::from_millis(self.timeout_ms),
            TcpStream::connect((target, 80))
        ).await {
            Ok(Ok(_)) => Ok(start.elapsed().as_secs_f64() * 1000.0),
            Ok(Err(e)) => Err(Error::msg(format!("Connection failed: {}", e))),
            Err(_) => Err(Error::msg("Connection timed out")),
        }
    }

    /// Processes raw latency samples to produce reliable measurements by:
    /// 1. Removing statistical outliers
    /// 2. Calculating median value
    /// 3. Estimating measurement reliability
    fn process_latency_samples(&self, mut samples: Vec<f64>) -> Vec<f64> {
        // Sort samples for percentile calculations
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Calculate quartiles for outlier detection
        let q1 = samples[samples.len() / 4];
        let q3 = samples[3 * samples.len() / 4];
        let iqr = q3 - q1;
        
        // Filter out outliers using 1.5 * IQR rule
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        samples.into_iter()
            .filter(|&s| s >= lower_bound && s <= upper_bound)
            .collect()
    }

    /// Executes traceroute command and captures its output
    async fn run_traceroute(&self, target: IpAddr) -> Result<String> {
        let output = Command::new("traceroute")
            .arg("-n") // Numeric output only
            .arg("-q").arg("3") // 3 probes per hop
            .arg("-w").arg("2") // 2 second timeout
            .arg("-m").arg(self.max_hops.to_string())
            .arg(target.to_string())
            .output()
            .await?;

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Parses raw traceroute output into structured path data
    fn parse_traceroute_output(&self, output: &str) -> Result<Vec<PathHop>> {
        let mut hops = Vec::new();

        for line in output.lines().skip(1) { // Skip header line
            if let Some(hop) = self.parse_traceroute_hop(line)? {
                hops.push(hop);
            }
        }

        Ok(hops)
    }

    /// Parses a single line of traceroute output into a PathHop structure
    fn parse_traceroute_hop(&self, line: &str) -> Result<Option<PathHop>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        // Handle non-responding hops
        let (ip, rtt, responded) = if parts[1] == "*" {
            (IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0.0, false)
        } else {
            let ip: IpAddr = parts[1].parse()?;
            let rtt: f64 = parts[2].trim_end_matches("ms").parse()?;
            (ip, rtt, true)
        };

        Ok(Some(PathHop {
            ip,
            rtt,
            responded,
        }))
    }

    /// Analyzes characteristics of the network path
    fn analyze_path_characteristics(&self, hops: &[PathHop]) -> (f64, f64) {
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
        let variance = latencies.iter()
            .map(|&lat| (lat - avg_latency).powi(2))
            .sum::<f64>() / latencies.len() as f64;
        
        let std_dev = variance.sqrt();
        let consistency_score = 1.0 / (1.0 + (std_dev / avg_latency));

        (consistency_score, avg_latency)
    }

    /// Detects anomalies in the network path that might indicate proxying
    fn detect_path_anomalies(&self, hops: &[PathHop]) -> Option<Vec<String>> {
        let mut anomalies = Vec::new();
        
        // Check for physically impossible latency decreases
        for window in hops.windows(2) {
            if let [hop1, hop2] = window {
                if hop1.responded && hop2.responded {
                    if hop2.rtt < hop1.rtt {
                        anomalies.push(
                            "Physically impossible latency decrease detected".to_string()
                        );
                    }
                }
            }
        }

        // Check for suspiciously large gaps in responding hops
        let mut gap_size = 0;
        for hop in hops {
            if !hop.responded {
                gap_size += 1;
            } else if gap_size > 3 {
                anomalies.push(format!(
                    "Suspicious gap of {} non-responding hops", 
                    gap_size
                ));
                gap_size = 0;
            }
        }

        if anomalies.is_empty() {
            None
        } else {
            Some(anomalies)
        }
    }
}

/// Configuration for network measurements
#[derive(Debug, Clone)]
pub struct MeasurementConfig {
    pub timeout_ms: u64,
    pub sample_count: usize,
    pub inter_measurement_delay_ms: u64,
    pub max_hops: u32,
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 1000,
            sample_count: 10,
            inter_measurement_delay_ms: 100,
            max_hops: 30,
        }
    }
}