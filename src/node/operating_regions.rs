use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Debug)]
pub enum RegionError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    ValidationError(String),
}

// Represents a city-based region where validators can operate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityRegion {
    pub city: String,
    pub jurisdiction_country: String,
    pub jurisdiction_state: String,
    pub flag: String,
    pub region_code: String,
    pub internet_exchange: String,
}

// Container for different types of regions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionTypes {
    // As we expand, we can add more region types here
    pub city: HashMap<String, CityRegion>,
}

// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub regions: RegionTypes,
}

impl RegionConfig {
    pub fn load() -> Result<Self, RegionError> {
        // Look for the configuration file in the config directory
        let config_path = PathBuf::from("config/regions.toml");
        
        let contents = fs::read_to_string(config_path)
            .map_err(|e| RegionError::IoError(e))?;
            
        let config: RegionConfig = toml::from_str(&contents)
            .map_err(|e| RegionError::ParseError(e))?;
            
        // Validate the configuration before returning
        config.validate()?;
        
        Ok(config)
    }

    fn validate(&self) -> Result<(), RegionError> {
        // Ensure we have at least one city region defined
        if self.regions.city.is_empty() {
            return Err(RegionError::ValidationError(
                "At least one city region must be defined".to_string()
            ));
        }

        // Validate each city region
        for (id, region) in &self.regions.city {
            if region.city.is_empty() {
                return Err(RegionError::ValidationError(
                    format!("City region {} must have a name", id)
                ));
            }
            
            if region.flag.is_empty() {
                return Err(RegionError::ValidationError(
                    format!("City region {} must have a flag emoji", id)
                ));
            }
        }

        Ok(())
    }

    // Helper method to format region information for display
    pub fn get_city_display(&self, region_id: &str) -> Option<String> {
        self.regions.city.get(region_id).map(|region| {
            format!(
                "{} {} ({}, {})",
                region.flag,
                region.city,
                region.jurisdiction_state,
                region.jurisdiction_country
            )
        })
    }
}

// Error handling implementations
impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionError::IoError(e) => write!(f, "IO error: {}", e),
            RegionError::ParseError(e) => write!(f, "Parse error: {}", e),
            RegionError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for RegionError {}

impl From<std::io::Error> for RegionError {
    fn from(error: std::io::Error) -> Self {
        RegionError::IoError(error)
    }
}

impl From<toml::de::Error> for RegionError {
    fn from(error: toml::de::Error) -> Self {
        RegionError::ParseError(error)
    }
}
