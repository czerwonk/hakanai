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
            assert!(result.is_ok(), "Expected {} to be valid", code);
            assert_eq!(result.unwrap().as_str(), code);
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
        let country_code = CountryCode::new("US").unwrap();
        assert_eq!(country_code.as_str(), "US");
    }

    #[test]
    fn test_from_string_conversion() {
        let country_code = CountryCode::new("DE").unwrap();
        let string_value: String = country_code.into();
        assert_eq!(string_value, "DE");
    }

    #[test]
    fn test_try_from_string_valid() {
        let result = CountryCode::try_from("CA".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "CA");
    }

    #[test]
    fn test_try_from_string_invalid() {
        let result = CountryCode::try_from("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_equality() {
        let code1 = CountryCode::new("US").unwrap();
        let code2 = CountryCode::new("US").unwrap();
        let code3 = CountryCode::new("DE").unwrap();

        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_clone() {
        let original = CountryCode::new("JP").unwrap();
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.as_str(), cloned.as_str());
    }

    #[test]
    fn test_serde_serialization() {
        let country_code = CountryCode::new("FR").unwrap();
        let json = serde_json::to_string(&country_code).unwrap();
        assert_eq!(json, "\"FR\"");
    }

    #[test]
    fn test_serde_deserialization_valid() {
        let json = "\"GB\"";
        let country_code: CountryCode = serde_json::from_str(json).unwrap();
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
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error
                .message
                .contains("CountryCode must be a 2-letter uppercase ISO 3166-1 alpha-2 code")
        );
    }

    #[test]
    fn test_from_str_valid() {
        let country_code: CountryCode = "US".parse().unwrap();
        assert_eq!(country_code.as_str(), "US");
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<CountryCode, _> = "invalid".parse();
        assert!(result.is_err());
    }
}
