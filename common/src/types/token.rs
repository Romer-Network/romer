/// Represents a token in the RØMER network
#[derive(Debug, Clone)]
pub struct Token {
    /// Unique identifier for the token
    pub id: String,
    
    /// Human-readable name of the token
    pub name: String,
    
    /// Token's trading symbol
    pub symbol: String,
    
    /// Number of decimal places for token precision
    pub decimals: u8,
    
    /// Reference to the organization that issued this token
    pub issuer_id: String,
    
    /// Total supply of the token (as raw units - actual value is total_supply / 10^decimals)
    pub total_supply: u128,
    
    /// Timestamp when the token was created (Unix timestamp in seconds)
    pub created_at: u64,
}

impl Token {
    /// Creates a new token with the current timestamp
    pub fn new(
        id: String,
        name: String,
        symbol: String,
        decimals: u8,
        issuer_id: String,
        total_supply: u128,
    ) -> Self {
        // Get current Unix timestamp in seconds
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            name,
            symbol,
            decimals,
            issuer_id,
            total_supply,
            created_at: now,
        }
    }

    /// Validates the token's data
    pub fn validate(&self) -> Result<(), String> {
        // Ensure ID is not empty
        if self.id.is_empty() {
            return Err("Token ID cannot be empty".to_string());
        }

        // Ensure name is not empty
        if self.name.is_empty() {
            return Err("Token name cannot be empty".to_string());
        }

        // Ensure symbol is not empty
        if self.symbol.is_empty() {
            return Err("Token symbol cannot be empty".to_string());
        }

        // Ensure decimals is within reasonable range (0-18)
        if self.decimals > 18 {
            return Err("Decimals must be 18 or less".to_string());
        }

        // Ensure issuer ID is not empty
        if self.issuer_id.is_empty() {
            return Err("Issuer ID cannot be empty".to_string());
        }

        Ok(())
    }

    /// Gets the actual token amount considering decimals
    pub fn get_actual_amount(&self, raw_amount: u128) -> f64 {
        let divisor = 10_u128.pow(self.decimals as u32) as f64;
        raw_amount as f64 / divisor
    }

    /// Gets the raw token amount from an actual amount
    pub fn get_raw_amount(&self, actual_amount: f64) -> Option<u128> {
        let multiplier = 10_u128.pow(self.decimals as u32);
        let raw = (actual_amount * multiplier as f64) as u128;
        
        // Check for overflow
        if self.get_actual_amount(raw) != actual_amount {
            return None;
        }
        
        Some(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_amount_conversion() {
        let token = Token::new(
            "test".to_string(),
            "Test Token".to_string(),
            "TEST".to_string(),
            6,  // 6 decimals
            "issuer1".to_string(),
            1_000_000_000_000,  // 1 million tokens with 6 decimals
        );

        // Test conversion from raw to actual
        assert_eq!(token.get_actual_amount(1_000_000), 1.0);
        assert_eq!(token.get_actual_amount(500_000), 0.5);

        // Test conversion from actual to raw
        assert_eq!(token.get_raw_amount(1.0), Some(1_000_000));
        assert_eq!(token.get_raw_amount(0.5), Some(500_000));
    }
}