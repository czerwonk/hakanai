use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::errors::ValidationError;

/// A struct representing a country code following the ISO 3166-1 alpha-2 standard.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct CountryCode(String);

impl CountryCode {
    pub fn new(code: &str) -> Result<Self, ValidationError> {
        if code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(CountryCode(code.to_string()))
        } else {
            Err(ValidationError::new(
                "CountryCode must be a 2-letter uppercase ISO 3166-1 alpha-2 code",
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<CountryCode> for String {
    fn from(code: CountryCode) -> Self {
        code.0
    }
}

impl TryFrom<String> for CountryCode {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CountryCode::new(&value)
    }
}

impl Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CountryCode {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CountryCode::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_country_codes() {
        let valid_codes = ["US", "DE", "CA", "GB", "FR", "JP", "AU"];

        for code in valid_codes {
            let result = CountryCode::new(code);
            assert_eq!(
                result
                    .expect(&format!("Expected {code} to be valid"))
                    .as_str(),
                code
            );
        }
    }

    #[test]
    fn test_invalid_length() {
        let invalid_codes = ["", "U", "USA", "DEUT", "GERMANY"];

        for code in invalid_codes {
            let result = CountryCode::new(code);
            assert!(result.is_err(), "Expected {} to be invalid", code);
        }
    }

    #[test]
    fn test_invalid_case() {
        let invalid_codes = ["us", "De", "ca", "gb"];

        for code in invalid_codes {
            let result = CountryCode::new(code);
            assert!(result.is_err(), "Expected {} to be invalid", code);
        }
    }

    #[test]
    fn test_invalid_characters() {
        let invalid_codes = ["U1", "D@", "C-", "G B", "12", "@@"];

        for code in invalid_codes {
            let result = CountryCode::new(code);
            assert!(result.is_err(), "Expected {} to be invalid", code);
        }
    }

    #[test]
    fn test_as_str() {
        let country_code = CountryCode::new("US").expect("Failed to parse valid country code");
        assert_eq!(country_code.as_str(), "US");
    }

    #[test]
    fn test_from_string_conversion() {
        let country_code = CountryCode::new("DE").expect("Failed to parse valid country code");
        let string_value: String = country_code.into();
        assert_eq!(string_value, "DE");
    }

    #[test]
    fn test_try_from_string_valid() -> Result<(), Box<dyn std::error::Error>> {
        let result = CountryCode::try_from("CA".to_string());
        assert_eq!(result?.as_str(), "CA");
        Ok(())
    }

    #[test]
    fn test_try_from_string_invalid() {
        let result = CountryCode::try_from("invalid".to_string());
        assert!(
            result.is_err(),
            "Expected error for invalid country code, got: {:?}",
            result
        );
    }

    #[test]
    fn test_equality() {
        let code1 = CountryCode::new("US").expect("Failed to parse valid country code");
        let code2 = CountryCode::new("US").expect("Failed to parse valid country code");
        let code3 = CountryCode::new("DE").expect("Failed to parse valid country code");

        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_clone() {
        let original = CountryCode::new("JP").expect("Failed to parse valid country code");
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.as_str(), cloned.as_str());
    }

    #[test]
    fn test_serde_serialization() {
        let country_code = CountryCode::new("FR").expect("Failed to parse valid country code");
        let json = serde_json::to_string(&country_code).expect("Failed to serialize in json");
        assert_eq!(json, "\"FR\"");
    }

    #[test]
    fn test_serde_deserialization_valid() {
        let json = "\"GB\"";
        let country_code: CountryCode =
            serde_json::from_str(json).expect("Failed to deserialize from json");
        assert_eq!(country_code.as_str(), "GB");
    }

    #[test]
    fn test_serde_deserialization_invalid() {
        let invalid_json_values = ["\"usa\"", "\"G\"", "\"123\"", "\"g b\""];

        for json in invalid_json_values {
            let result: Result<CountryCode, _> = serde_json::from_str(json);
            assert!(result.is_err(), "Expected {} to fail deserialization", json);
        }
    }

    #[test]
    fn test_error_message() {
        let result = CountryCode::new("invalid");
        assert!(
            result.is_err(),
            "Expected error for invalid country code, got: {:?}",
            result
        );
        let error = result.unwrap_err();
        assert!(
            error
                .message
                .contains("CountryCode must be a 2-letter uppercase ISO 3166-1 alpha-2 code")
        );
    }

    #[test]
    fn test_from_str_valid() {
        let country_code: CountryCode = "US".parse().expect("Failed to parse valid country code");
        assert_eq!(country_code.as_str(), "US");
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<CountryCode, _> = "invalid".parse();
        assert!(
            result.is_err(),
            "Expected error for invalid country code parse, got: {:?}",
            result
        );
    }
}
