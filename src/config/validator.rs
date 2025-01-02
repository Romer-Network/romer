use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::node::operating_regions::RegionConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorConfig {
    pub city: String,
}

#[derive(Debug)]
pub enum ValidatorConfigError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    RegionNotFound(String),
}

impl std::fmt::Display for ValidatorConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidatorConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ValidatorConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            ValidatorConfigError::RegionNotFound(city) => {
                write!(f, "City '{}' not found in authorized regions", city)
            }
        }
    }
}

impl std::error::Error for ValidatorConfigError {}

impl ValidatorConfig {
    /// Loads the validator configuration from the config directory
    pub fn load_validator_config() -> Result<Self, ValidatorConfigError> {
        // Load region configuration first
        let region_config = RegionConfig::load().map_err(|e| {
            ValidatorConfigError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load region config: {}", e),
            ))
        })?;

        // Load validator configuration
        let config_path = PathBuf::from("config/validator.toml");
        let contents =
            fs::read_to_string(config_path).map_err(|e| ValidatorConfigError::IoError(e))?;

        let config: ValidatorConfig =
            toml::from_str(&contents).map_err(|e| ValidatorConfigError::ParseError(e))?;

        // Validate the city exists in authorized regions
        let city_key = config.city.to_lowercase().replace(" ", "-");
        if !region_config.regions.city.contains_key(&city_key) {
            return Err(ValidatorConfigError::RegionNotFound(config.city.clone()));
        }

        Ok(config)
    }

    /// Determines the path to the validator configuration file
    fn get_validator_config_path() -> Result<PathBuf, String> {
        // Start with the current directory
        let mut path = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        // Add the config directory
        path.push("config");

        // Check if config directory exists
        if !path.exists() {
            return Err(
                "Config directory not found. Please ensure the ./config directory exists"
                    .to_string(),
            );
        }

        // Add validator.toml
        path.push("validator.toml");

        // Check if the configuration file exists
        if !path.exists() {
            return Err("validator.toml not found in config directory. Please ensure ./config/validator.toml exists".to_string());
        }

        Ok(path)
    }

    /// Loads and validates the configuration from a specific path
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        // Read the configuration file
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read validator configuration: {}", e))?;

        // Parse the TOML content
        let config: ValidatorConfig = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse validator configuration: {}", e))?;

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Validates the configuration values
    fn validate(&self) -> Result<(), String> {
        // Check that city is not empty
        if self.city.trim().is_empty() {
            return Err("City cannot be empty in validator configuration".to_string());
        }

        // City should not contain special characters
        if !self
            .city
            .chars()
            .all(|c| c.is_alphabetic() || c.is_whitespace())
        {
            return Err("City name should only contain letters and spaces".to_string());
        }

        Ok(())
    }
}
