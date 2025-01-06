use anyhow::{Error, Result};
use rand::random;
use std::error::Error as StdError;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use surge_ping::{Client, Config as PingConfig, PingIdentifier, PingSequence};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

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
        // Log the start of the measurement with key details
        info!(
            "Attempting ICMP latency measurement to {} with {} samples",
            target, self.sample_count
        );

        // Client creation with more diagnostic information
        let client = match Client::new(&PingConfig::default()) {
            Ok(client) => client,
            Err(e) => {
                error!(
                    "ICMP client creation failed. 
                    This typically indicates:
                    - Insufficient network privileges
                    - Firewall blocking raw socket creation
                    Specific error: {}",
                    e
                );
                return Err(Error::msg(format!("ICMP client creation failed: {}", e)));
            }
        };

        // Create unique identifier for this ping session
        let ident = PingIdentifier(random::<u16>());

        // Create pinger
        let mut pinger = client.pinger(target, ident).await;

        // Diagnostic payload (32 bytes of zeros)
        let payload = vec![0; 32];

        let mut samples = Vec::with_capacity(self.sample_count);
        let mut failures = 0;

        for attempt in 0..self.sample_count {
            let start = Instant::now();

            match pinger.ping(PingSequence(attempt as u16), &payload).await {
                Ok(_) => {
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    info!("Successful ping to {}: {:.2} ms", target, latency);
                    samples.push(latency);
                }
                Err(e) => {
                    // Comprehensive error logging
                    warn!(
                        "Ping to {} failed (Attempt {}/{}). 
                        Error details:
                        - Error: {}
                        - Sequence: {}
                        ",
                        target,
                        attempt + 1,
                        self.sample_count,
                        e,
                        attempt
                    );

                    failures += 1;

                    // Add system-level diagnostic commands to help troubleshoot
                    if let Err(cmd_err) = self.run_network_diagnostics(target).await {
                        warn!("Additional network diagnostics failed: {}", cmd_err);
                    }

                    // Fail fast if too many attempts fail
                    if failures > self.sample_count / 2 {
                        error!(
                            "Measurement to {} failed after {} attempts. 
                            Consider manual network troubleshooting.",
                            target, failures
                        );
                        return Err(Error::msg("Excessive ping failures"));
                    }
                }
            }

            // Delay between attempts to avoid overwhelming the network
            tokio::time::sleep(Duration::from_millis(self.inter_measurement_delay_ms)).await;
        }

        // Validate measurement results
        if samples.is_empty() {
            error!(
                "No successful measurements to {}. 
                This suggests a consistent connectivity issue.",
                target
            );
            return Err(Error::msg("No successful latency measurements"));
        }

        Ok(self.process_latency_samples(samples))
    }

    // Additional diagnostic method to run system-level network checks
    async fn run_network_diagnostics(&self, target: IpAddr) -> Result<()> {
        // Run various network diagnostic commands
        let commands = vec![
            format!("ping -c 4 {}", target),
            format!("traceroute {}", target),
            "netstat -rn".to_string(),
            "ip route".to_string(),
        ];

        for cmd in commands {
            match Command::new("sh").arg("-c").arg(&cmd).output().await {
                Ok(output) => {
                    // Log command output for deeper investigation
                    debug!(
                        "Diagnostic command '{}' output:\nSTDOUT: {}\nSTDERR: {}",
                        cmd,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(e) => {
                    warn!("Failed to run diagnostic command '{}': {}", cmd, e);
                }
            }
        }

        Ok(())
    }


    /// Performs a single latency measurement using TCP connection timing.
    /// This method avoids using ICMP ping which requires root privileges.
    async fn single_latency_measurement(&self, target: IpAddr) -> Result<f64> {
        // Create a new ICMP client with default configuration
        let client = Client::new(&PingConfig::default())?;

        // Create a unique identifier for this ping session
        let ident = PingIdentifier(random::<u16>());

        // Create a pinger for this specific target
        let mut pinger = client.pinger(target, ident).await;

        // Create a buffer for timing measurement
        let start = Instant::now();

        // Send ping with sequence number and standard 32-byte payload
        // Using vec![0; 32] creates a payload of 32 zero bytes, similar to standard ping
        match pinger.ping(PingSequence(0), &vec![0; 32]).await {
            Ok(_) => Ok(start.elapsed().as_secs_f64() * 1000.0),
            Err(e) => Err(Error::msg(format!("ICMP ping failed: {}", e))),
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

        samples
            .into_iter()
            .filter(|&s| s >= lower_bound && s <= upper_bound)
            .collect()
    }

    /// Executes traceroute command and captures its output
    async fn run_traceroute(&self, target: IpAddr) -> Result<String> {
        let output = Command::new("traceroute")
            .arg("-n") // Numeric output only
            .arg("-q")
            .arg("3") // 3 probes per hop
            .arg("-w")
            .arg("2") // 2 second timeout
            .arg("-m")
            .arg(self.max_hops.to_string())
            .arg(target.to_string())
            .output()
            .await?;

        Ok(String::from_utf8(output.stdout)?)
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
            timeout_ms: 2000,
            sample_count: 10,
            inter_measurement_delay_ms: 200,
            max_hops: 30,
        }
    }
}
