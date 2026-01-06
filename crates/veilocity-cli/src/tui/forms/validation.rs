//! Form Validation
//!
//! Input validation for deposit and withdraw forms.

use std::fmt;

/// Validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidNumber,
    AmountMustBePositive,
    AmountTooSmall,
    AmountTooLarge,
    InvalidAddress,
    AddressTooShort,
    AddressMissingPrefix,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidNumber => write!(f, "Invalid number"),
            ValidationError::AmountMustBePositive => write!(f, "Amount must be positive"),
            ValidationError::AmountTooSmall => write!(f, "Amount too small (min: 0.000001 MNT)"),
            ValidationError::AmountTooLarge => write!(f, "Amount too large (max: 1,000,000 MNT)"),
            ValidationError::InvalidAddress => write!(f, "Invalid Ethereum address"),
            ValidationError::AddressTooShort => write!(f, "Address must be 42 characters (0x + 40 hex)"),
            ValidationError::AddressMissingPrefix => write!(f, "Address must start with 0x"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate an amount input
pub fn validate_amount(input: &str) -> Result<f64, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::InvalidNumber);
    }

    let amount: f64 = input.parse().map_err(|_| ValidationError::InvalidNumber)?;

    if amount <= 0.0 {
        return Err(ValidationError::AmountMustBePositive);
    }

    if amount < 0.000001 {
        return Err(ValidationError::AmountTooSmall);
    }

    if amount > 1_000_000.0 {
        return Err(ValidationError::AmountTooLarge);
    }

    Ok(amount)
}

/// Validate an Ethereum address
pub fn validate_address(input: &str) -> Result<String, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::InvalidAddress);
    }

    if !input.starts_with("0x") && !input.starts_with("0X") {
        return Err(ValidationError::AddressMissingPrefix);
    }

    if input.len() != 42 {
        return Err(ValidationError::AddressTooShort);
    }

    // Check all characters after 0x are hex digits
    let hex_part = &input[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidAddress);
    }

    // Normalize to lowercase
    Ok(input.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_amount_valid() {
        assert!(validate_amount("1.0").is_ok());
        assert!(validate_amount("0.5").is_ok());
        assert!(validate_amount("100").is_ok());
        assert!(validate_amount("0.000001").is_ok());
    }

    #[test]
    fn test_validate_amount_invalid() {
        assert!(validate_amount("").is_err());
        assert!(validate_amount("-1").is_err());
        assert!(validate_amount("0").is_err());
        assert!(validate_amount("abc").is_err());
        assert!(validate_amount("0.0000001").is_err()); // Too small
        assert!(validate_amount("10000000").is_err()); // Too large
    }

    #[test]
    fn test_validate_address_valid() {
        assert!(validate_address("0x1234567890abcdef1234567890abcdef12345678").is_ok());
        assert!(validate_address("0xABCDEF1234567890ABCDEF1234567890ABCDEF12").is_ok());
    }

    #[test]
    fn test_validate_address_invalid() {
        assert!(validate_address("").is_err());
        assert!(validate_address("1234567890abcdef1234567890abcdef12345678").is_err()); // No 0x
        assert!(validate_address("0x1234").is_err()); // Too short
        assert!(validate_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG").is_err()); // Invalid hex
    }
}
