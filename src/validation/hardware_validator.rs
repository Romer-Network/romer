use std::env;
use std::process::Command;
use anyhow::{Context, Result};
use tracing::info;

/// Represents different virtualization types that we might detect.
/// This helps us clearly categorize the execution environment of the node.
#[derive(Debug, Clone, PartialEq)]
pub enum VirtualizationType {
    /// Indicates the system is running on physical hardware
    Physical,
    /// Represents a specific virtualization technology with its name
    Virtual(String),
}

/// Represents the operating system type. We need this to determine
/// which validation strategies to use, as each OS has different
/// methods for detecting virtualization.
#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

/// The main hardware detection system. This struct serves as the entry point
/// for all hardware-related validation operations.
pub struct HardwareDetector;

impl HardwareDetector {
    /// Detects the current operating system using conditional compilation.
    /// This approach ensures we get the correct OS at compile time rather
    /// than having to detect it at runtime.
    pub fn detect_os() -> OperatingSystem {
        // Use cfg attributes to determine the OS at compile time
        #[cfg(windows)]
        {
            info!("Operating System: Windows");
            return OperatingSystem::Windows;
        }

        #[cfg(target_os = "macos")]
        {
            info!("Operating System: MacOS");
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

    /// Detects virtualization across different operating systems.
    /// Returns a Result with either VirtualizationType or an error with context.
    pub fn detect_virtualization() -> Result<VirtualizationType> {
        // Route to the appropriate detection method based on OS
        match Self::detect_os() {
            OperatingSystem::Windows => Self::detect_windows_virtualization(),
            OperatingSystem::MacOS => Self::detect_macos_virtualization(),
            OperatingSystem::Linux => Self::detect_linux_virtualization(),
            OperatingSystem::Unknown => Ok(VirtualizationType::Physical), // Conservative default
        }
    }

    /// Windows-specific virtualization detection.
    /// Uses both environment variables and WMI queries to detect virtualization.
    fn detect_windows_virtualization() -> Result<VirtualizationType> {
        // Check environment variables first (faster)
        if env::var("SYSTEMTYPE").map_or(false, |v| v == "VIRTUAL") {
            return Ok(VirtualizationType::Virtual("Generic Virtual".to_string()));
        }

        // Use WMI to check system model
        let output = Command::new("wmic")
            .args(&["computersystem", "get", "model"])
            .output()
            .context("Failed to execute wmic command")?;

        let model_str = String::from_utf8(output.stdout)
            .context("Failed to parse wmic command output")?;

        if model_str.contains("VMware") {
            Ok(VirtualizationType::Virtual("VMware".to_string()))
        } else {
            Ok(VirtualizationType::Physical)
        }
    }

    /// MacOS-specific virtualization detection.
    /// Uses system profiler to gather hardware information.
    fn detect_macos_virtualization() -> Result<VirtualizationType> {
        let output = Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
            .context("Failed to execute system_profiler command")?;

        let hardware_info = String::from_utf8(output.stdout)
            .context("Failed to parse system_profiler output")?;

        // Check for known virtualization markers
        if hardware_info.contains("VMware") {
            Ok(VirtualizationType::Virtual("VMware".to_string()))
        } else if hardware_info.contains("Parallels") {
            Ok(VirtualizationType::Virtual("Parallels".to_string()))
        } else {
            Ok(VirtualizationType::Physical)
        }
    }

    /// Linux-specific virtualization detection.
    /// Uses multiple detection methods in sequence, falling back to simpler
    /// methods if more sophisticated ones fail.
    fn detect_linux_virtualization() -> Result<VirtualizationType> {
        // Try systemd-detect-virt first
        if let Ok(output) = Command::new("systemd-detect-virt").output() {
            if output.status.success() {
                let virt_type = String::from_utf8(output.stdout)
                    .context("Failed to parse systemd-detect-virt output")?
                    .trim()
                    .to_string();
                
                if virt_type != "none" {
                    return Ok(VirtualizationType::Virtual(virt_type));
                }
            }
        }

        // Try DMI detection as fallback
        if let Ok(output) = Command::new("dmidecode")
            .arg("-t")
            .arg("system")
            .output()
        {
            let output_str = String::from_utf8(output.stdout)
                .context("Failed to parse dmidecode output")?;
            
            if output_str.contains("VMware") || output_str.contains("Virtual") {
                return Ok(VirtualizationType::Virtual("VMware".to_string()));
            }
        }

        // Check environment variables as last resort
        if env::var("VIRTUAL_ENV").is_ok() {
            return Ok(VirtualizationType::Virtual("Python Virtual Env".to_string()));
        }
        if env::var("CONTAINER").is_ok() {
            return Ok(VirtualizationType::Virtual("Container".to_string()));
        }
        if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            return Ok(VirtualizationType::Virtual("Kubernetes".to_string()));
        }

        // If no virtualization detected, assume physical
        Ok(VirtualizationType::Physical)
    }
}

/// Unit tests for hardware detection
#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_virtualization_detection() {
        // We expect this to complete without panicking
        let result = HardwareDetector::detect_virtualization();
        assert!(result.is_ok(), "Virtualization detection should not fail");
    }
}