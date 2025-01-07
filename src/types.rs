// src/types.rs

use std::fmt;

/// Represents a validator's geographic location with validation logic
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorLocation {
    /// Latitude in degrees (-90 to 90)
    latitude: f64,
    
    /// Longitude in degrees (-180 to 180)
    longitude: f64,
}

impl ValidatorLocation {
    /// Creates a new ValidatorLocation after validating the coordinates.
    /// Returns an error if the coordinates are outside their valid ranges.
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, LocationError> {
        // Validate latitude
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(LocationError::InvalidLatitude(latitude));
        }
        
        // Validate longitude
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(LocationError::InvalidLongitude(longitude));
        }

        Ok(Self {
            latitude,
            longitude,
        })
    }

    /// Returns the latitude value
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// Returns the longitude value
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// Converts the location to a geo::Point for geometric calculations
    pub fn to_point(&self) -> geo::Point<f64> {
        geo::Point::new(self.longitude, self.latitude)
    }
}

/// Represents errors that can occur when working with validator locations
#[derive(Debug, thiserror::Error)]
pub enum LocationError {
    #[error("Invalid latitude {0}: must be between -90 and 90 degrees")]
    InvalidLatitude(f64),
    
    #[error("Invalid longitude {0}: must be between -180 and 180 degrees")]
    InvalidLongitude(f64),
}

impl fmt::Display for ValidatorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.latitude, self.longitude)
    }
}