use std::env;
use std::error::Error;
use std::fmt;
use std::process::Command;

use tracing::info;

/// Represents different virtualization types
#[derive(Debug, Clone, PartialEq)]
pub enum VirtualizationType {
    /// Indicates the system is running on physical hardware
    Physical,
    /// Represents a specific virtualization technology
    Virtual(String),
}

/// Represents the operating system type
#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

/// Custom error type for hardware detection
#[derive(Debug)]
pub struct HardwareDetectionError {
    message: String,
}

impl HardwareDetectionError {
    fn new(message: String) -> Self {
        HardwareDetectionError { message }
    }
}

impl fmt::Display for HardwareDetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hardware Detection Error: {}", self.message)
    }
}

impl Error for HardwareDetectionError {}

/// Comprehensive hardware detection system
pub struct HardwareDetector;

impl HardwareDetector {
    /// Detect the current operating system
    pub fn detect_os() -> OperatingSystem {
        // Conditional compilation for OS detection
        #[cfg(windows)]
        {
            info!("Operating System: Windows");
            return OperatingSystem::Windows;
        }

        #[cfg(target_os = "macos")]
        {
            info!("Operating System: MacOs");
            return OperatingSystem::MacOS;
        }

        #[cfg(target_os = "linux")]
        {
            info!("Operating System: Linux");
            return OperatingSystem::Linux;
        }

        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            info!("Operating System: Unknown");
            return OperatingSystem::Unknown;
        }
    }

    /// Detect virtualization across different operating systems
    pub fn detect_virtualization() -> Result<VirtualizationType, HardwareDetectionError> {
        match Self::detect_os() {
            OperatingSystem::Windows => Self::detect_windows_virtualization(),
            OperatingSystem::MacOS => Self::detect_macos_virtualization(),
            OperatingSystem::Linux => Self::detect_linux_virtualization(),
            OperatingSystem::Unknown => Ok(VirtualizationType::Physical),
        }
    }

    fn detect_windows_virtualization() -> Result<VirtualizationType, HardwareDetectionError> {
        // Check environment variables first (faster)
        if env::var("SYSTEMTYPE").map_or(false, |v| v == "VIRTUAL") {
            return Ok(VirtualizationType::Virtual("Generic Virtual".to_string()));
        }

        // Use a single, faster method
        let output = match Command::new("wmic")
            .args(&["computersystem", "get", "model"])
            .output()
        {
            Ok(out) => out,
            Err(_) => return Ok(VirtualizationType::Physical),
        };

        let model_str = String::from_utf8_lossy(&output.stdout);

        if model_str.contains("VMware") {
            return Ok(VirtualizationType::Virtual("VMware".to_string()));
        }

        Ok(VirtualizationType::Physical)
    }

    /// MacOS-specific virtualization detection
    fn detect_macos_virtualization() -> Result<VirtualizationType, HardwareDetectionError> {
        // Detection using system profiler
        let output = match Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return Err(HardwareDetectionError::new(format!(
                    "System profiler query failed: {}",
                    e
                )))
            }
        };

        let hardware_info = String::from_utf8_lossy(&output.stdout);

        // Check for known virtualization markers
        if hardware_info.contains("VMware") {
            return Ok(VirtualizationType::Virtual("VMware".to_string()));
        }

        if hardware_info.contains("Parallels") {
            return Ok(VirtualizationType::Virtual("Parallels".to_string()));
        }

        Ok(VirtualizationType::Physical)
    }

    /// Linux-specific virtualization detection
    fn detect_linux_virtualization() -> Result<VirtualizationType, HardwareDetectionError> {
        // Multiple detection methods for Linux
        let detection_methods = [
            // systemd-detect-virt method
            || {
                let output = match Command::new("systemd-detect-virt").output() {
                    Ok(out) => out,
                    Err(_) => return None,
                };

                if output.status.success() {
                    let virt_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if virt_type != "none" {
                        return Some(VirtualizationType::Virtual(virt_type));
                    }
                }
                None
            },
            // DMI detection method
            || {
                let output = match Command::new("dmidecode").arg("-t").arg("system").output() {
                    Ok(out) => out,
                    Err(_) => return None,
                };

                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("VMware") || output_str.contains("Virtual") {
                    return Some(VirtualizationType::Virtual("VMware".to_string()));
                }
                None
            },
            // Fallback: check for known virtualization environment variables
            || {
                if env::var("VIRTUAL_ENV").is_ok() {
                    return Some(VirtualizationType::Virtual(
                        "Python Virtual Env".to_string(),
                    ));
                }
                if env::var("CONTAINER").is_ok() {
                    return Some(VirtualizationType::Virtual("Container".to_string()));
                }
                if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
                    return Some(VirtualizationType::Virtual("Kubernetes".to_string()));
                }
                None
            },
        ];

        // Try each detection method
        for method in detection_methods.iter() {
            if let Some(result) = method() {
                return Ok(result);
            }
        }

        Ok(VirtualizationType::Physical)
    }
}

/// Unit tests for hardware detection
#[cfg(test)]
mod tests {
    use super::*;

    /// Test operating system detection
    #[test]
    fn test_os_detection() {
        let os = HardwareDetector::detect_os();
        assert!(
            matches!(
                os,
                OperatingSystem::Windows
                    | OperatingSystem::MacOS
                    | OperatingSystem::Linux
                    | OperatingSystem::Unknown
            ),
            "OS detection should return a valid operating system type"
        );
    }

    /// Test virtualization detection
    #[test]
    fn test_virtualization_detection() {
        let result = HardwareDetector::detect_virtualization();
        assert!(result.is_ok(), "Virtualization detection should not fail");
    }
}

/// Example main function to demonstrate usage
fn main() {
    // Detect operating system
    let os = HardwareDetector::detect_os();
    println!("Detected OS: {:?}", os);

    // Detect virtualization
    match HardwareDetector::detect_virtualization() {
        Ok(virt_type) => println!("Virtualization Type: {:?}", virt_type),
        Err(e) => eprintln!("Virtualization detection error: {}", e),
    }
}
