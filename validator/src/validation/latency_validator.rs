use anyhow::{Error, Result};
use geo::{Point, HaversineDistance};
use std::time::{Duration, Instant};
use surge_ping::{Client, Config as PingConfig, PingIdentifier, PingSequence};
use rand::random;
use tracing::{info, warn};

// Physics constants
const SPEED_OF_LIGHT_KMS: f64 = 299_792.458; // Speed of light in km/s
const FIBER_OVERHEAD: f64 = 1.4; // Typical fiber route overhead factor
const PROCESSING_OVERHEAD_MS: f64 = 0.1; // Minimal processing overhead

/// Represents the result of a latency validation
#[derive(Debug, Clone)]
pub struct LatencyValidationResult {
    pub theoretical_min_ms: f64,
    pub measured_latency_ms: f64,
    pub is_valid: bool,
    pub details: String,
}

/// Configuration for latency measurements
#[derive(Debug, Clone)]
pub struct LatencyConfig {
    pub sample_count: usize,
    pub timeout_ms: u64,
    pub max_latency_ratio: f64,  // Maximum allowed ratio of measured/theoretical latency
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            sample_count: 10,
            timeout_ms: 2000,
            max_latency_ratio: 2.0,  // Allow up to 2.0x theoretical minimum
        }
    }
}

/// Core latency validation functionality
pub struct LatencyValidator {
    config: LatencyConfig,
}

impl LatencyValidator {
    pub fn new(config: LatencyConfig) -> Self {
        Self { config }
    }

    /// Validates the latency between two geographic points
    pub async fn validate_latency(
        &self,
        point_a: Point<f64>,
        point_b: Point<f64>, 
        target_ip: std::net::IpAddr,
    ) -> Result<LatencyValidationResult> {
        // Calculate theoretical minimum latency
        let theoretical_min = self.calculate_theoretical_minimum(point_a, point_b);
        
        // Measure actual latency
        let measured_latency = self.measure_latency(target_ip).await?;
        
        // Validate results
        let is_valid = measured_latency <= (theoretical_min * self.config.max_latency_ratio);
        
        let details = format!(
            "Theoretical minimum: {:.2}ms, Measured: {:.2}ms, Ratio: {:.2}",
            theoretical_min,
            measured_latency,
            measured_latency / theoretical_min
        );

        Ok(LatencyValidationResult {
            theoretical_min_ms: theoretical_min,
            measured_latency_ms: measured_latency,
            is_valid,
            details,
        })
    }

    /// Calculates theoretical minimum latency between two points based on
    /// speed of light through fiber optic cables
    fn calculate_theoretical_minimum(&self, point_a: Point<f64>, point_b: Point<f64>) -> f64 {
        // Calculate great circle distance
        let distance_km = point_a.haversine_distance(&point_b);
        
        // Calculate time for light to travel through fiber:
        // 1. Account for fiber path being longer than great circle (FIBER_OVERHEAD)
        // 2. Convert to round trip (multiply by 2)
        // 3. Add minimal processing overhead
        let theoretical_ms = (distance_km * FIBER_OVERHEAD * 2.0 / SPEED_OF_LIGHT_KMS) * 1000.0 
            + PROCESSING_OVERHEAD_MS;

        info!(
            "Theoretical minimum latency calculation:\n\
            Distance: {:.3}km\n\
            Minimum latency: {:.3}ms",
            distance_km, theoretical_ms
        );

        theoretical_ms
    }

    /// Measures actual network latency to a target IP
    async fn measure_latency(&self, target: std::net::IpAddr) -> Result<f64> {
        // Create ICMP client
        let client = Client::new(&PingConfig::default())?;
        
        // Create unique identifier for this ping session
        let ident = PingIdentifier(random::<u16>());
        
        // Create pinger
        let mut pinger = client.pinger(target, ident).await;
        
        // Standard payload
        let payload = vec![0; 32];
        
        let mut latencies = Vec::with_capacity(self.config.sample_count);
        let mut failures = 0;

        // Collect samples
        for sequence in 0..self.config.sample_count {
            let start = Instant::now();
            
            match tokio::time::timeout(
                Duration::from_millis(self.config.timeout_ms),
                pinger.ping(PingSequence(sequence as u16), &payload)
            ).await {
                Ok(Ok(_)) => {
                    let latency = start.elapsed().as_secs_f64() * 1000.0;
                    info!("Successful ping: {:.2}ms", latency);
                    latencies.push(latency);
                },
                Ok(Err(e)) => {
                    warn!("Ping failed: {}", e);
                    failures += 1;
                },
                Err(_) => {
                    warn!("Ping timed out");
                    failures += 1;
                }
            }

            // Small delay between pings
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Require at least 50% successful measurements
        if failures > self.config.sample_count / 2 {
            return Err(Error::msg(format!(
                "Too many failed measurements: {} out of {}",
                failures,
                self.config.sample_count
            )));
        }

        // Calculate median latency (more robust than mean)
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_idx = latencies.len() / 2;
        Ok(latencies[median_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theoretical_minimum() {
        let validator = LatencyValidator::new(LatencyConfig::default());
        
        // Test points 1000km apart
        let point_a = Point::new(0.0, 0.0);
        let point_b = Point::new(8.993216, 0.0); // Approximately 1000km at equator
        
        let min_latency = validator.calculate_theoretical_minimum(point_a, point_b);
        
        // Expected: 1000km * 1.4 * 2 / 299792.458 * 1000 + 0.1
        // Should be approximately 9.34ms
        assert!((min_latency - 9.34).abs() < 0.1);
    }
}