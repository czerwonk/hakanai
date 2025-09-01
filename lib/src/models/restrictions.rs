// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::CountryCode;
use crate::utils::hashing;

pub const PASSPHRASE_HEADER_NAME: &str = "X-Secret-Passphrase";

/// Represents access restrictions for a secret.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SecretRestrictions {
    /// IP addresses/ranges allowed to access the secret
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "crate::utils::ip::deserialize_ip_nets")]
    pub allowed_ips: Option<Vec<ipnet::IpNet>>,

    /// Geo-IP locations allowed to access the secret
    pub allowed_countries: Option<Vec<CountryCode>>,

    /// ASNs allowed to access the secret
    pub allowed_asns: Option<Vec<u32>>,

    /// Optional passphrase hash for additional security
    pub passphrase_hash: Option<String>,
}

impl SecretRestrictions {
    /// Sets the IPs allowed to access the secret
    pub fn with_allowed_ips(mut self, allowed_ips: Vec<ipnet::IpNet>) -> Self {
        self.allowed_ips = Some(allowed_ips);
        self
    }

    /// Sets the countries allowed to access the secret
    pub fn with_allowed_countries(mut self, allowed_countries: Vec<CountryCode>) -> Self {
        self.allowed_countries = Some(allowed_countries);
        self
    }

    /// Sets the countries allowed to access the secret
    pub fn with_allowed_asns(mut self, allowed_asns: Vec<u32>) -> Self {
        self.allowed_asns = Some(allowed_asns);
        self
    }

    /// Sets the required passhphrase to access the secret
    pub fn with_passphrase(mut self, passphrase: &[u8]) -> Self {
        let hash = hashing::sha256_hex_from_bytes(passphrase);
        self.passphrase_hash = Some(hash);
        self
    }

    /// Checks if any restrictions are set
    pub fn is_empty(&self) -> bool {
        let any_ips = self.allowed_ips.as_ref().is_some_and(|v| !v.is_empty());
        if any_ips {
            return false;
        }

        let any_countries = self
            .allowed_countries
            .as_ref()
            .is_some_and(|v| !v.is_empty());
        if any_countries {
            return false;
        }

        let any_asns = self.allowed_asns.as_ref().is_some_and(|v| !v.is_empty());
        if any_asns {
            return false;
        }

        if self.passphrase_hash.as_ref().is_some_and(|h| h.len() > 0) {
            return false;
        }

        true
    }
}

impl Display for SecretRestrictions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "No restrictions");
        }

        if let Some(ips) = &self.allowed_ips {
            let ip_strings: Vec<String> = ips.iter().map(|ip| ip.to_string()).collect();
            write!(f, "Allowed IPs: {}", ip_strings.join(", "))?;
        }

        if let Some(countries) = &self.allowed_countries {
            let country_strings: Vec<String> = countries
                .iter()
                .map(|country| country.to_string())
                .collect();
            write!(f, "Allowed Countries: {}", country_strings.join(", "))?;
        }

        if let Some(asns) = &self.allowed_asns {
            let asn_strings: Vec<String> = asns.iter().map(|ip| ip.to_string()).collect();
            write!(f, "Allowed ASNs: {}", asn_strings.join(", "))?;
        }

        if self.passphrase_hash.is_some() {
            write!(f, "Passphrase: ***")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CountryCode;
    use ipnet::IpNet;

    #[test]
    fn test_secret_restrictions_deserialization() {
        // Test with valid IP addresses and CIDR ranges
        let json = r#"{"allowed_ips": ["127.0.0.1", "192.168.1.0/24", "::1", "2001:db8::/32"]}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        let ips = restrictions.allowed_ips.unwrap();
        assert_eq!(ips.len(), 4);
        assert_eq!(ips[0].to_string(), "127.0.0.1/32");
        assert_eq!(ips[1].to_string(), "192.168.1.0/24");
        assert_eq!(ips[2].to_string(), "::1/128");
        assert_eq!(ips[3].to_string(), "2001:db8::/32");
        assert!(restrictions.allowed_countries.is_none());
        assert!(restrictions.allowed_asns.is_none());
    }

    #[test]
    fn test_secret_restrictions_deserialization_with_countries() {
        // Test with valid country codes
        let json = r#"{"allowed_countries": ["US", "DE", "CA"]}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        let countries = restrictions.allowed_countries.unwrap();
        assert_eq!(countries.len(), 3);
        assert_eq!(countries[0].as_str(), "US");
        assert_eq!(countries[1].as_str(), "DE");
        assert_eq!(countries[2].as_str(), "CA");
        assert!(restrictions.allowed_ips.is_none());
        assert!(restrictions.allowed_asns.is_none());
    }

    #[test]
    fn test_secret_restrictions_deserialization_with_asns() {
        // Test with valid country codes
        let json = r#"{"allowed_asns": [48821,202739]}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        let asns = restrictions.allowed_asns.unwrap();
        assert_eq!(asns.len(), 2);
        assert_eq!(asns[0], 48821);
        assert_eq!(asns[1], 202739);
        assert!(restrictions.allowed_ips.is_none());
        assert!(restrictions.allowed_countries.is_none());
    }

    #[test]
    fn test_secret_restrictions_deserialization_all() {
        // Test with both IPs and countries
        let json = r#"{"allowed_ips": ["192.168.1.0/24"], "allowed_countries": ["US", "DE"], "allowed_asns": [202739]}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        let ips = restrictions.allowed_ips.unwrap();
        assert_eq!(ips.len(), 1);
        assert_eq!(ips[0].to_string(), "192.168.1.0/24");

        let countries = restrictions.allowed_countries.unwrap();
        assert_eq!(countries.len(), 2);
        assert_eq!(countries[0].as_str(), "US");
        assert_eq!(countries[1].as_str(), "DE");

        let asns = restrictions.allowed_asns.unwrap();
        assert_eq!(asns.len(), 1);
        assert_eq!(asns[0], 202739);
    }

    #[test]
    fn test_secret_restrictions_deserialization_empty() {
        // Test with null allowed_ips
        let json = r#"{"allowed_ips": null}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());
        assert!(restrictions.allowed_countries.is_none());
        assert!(restrictions.allowed_asns.is_none());

        // Test with null allowed_countries
        let json = r#"{"allowed_countries": null}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());
        assert!(restrictions.allowed_countries.is_none());
        assert!(restrictions.allowed_asns.is_none());

        // Test with empty object
        let json = r#"{}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());
        assert!(restrictions.allowed_countries.is_none());
        assert!(restrictions.allowed_asns.is_none());
    }

    #[test]
    fn test_secret_restrictions_deserialization_invalid_ip() {
        let json = r#"{"allowed_ips": ["invalid-ip"]}"#;
        let result: std::result::Result<SecretRestrictions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Expected error for invalid IP, got: {:?}",
            result
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid IP address or CIDR notation")
        );
    }

    #[test]
    fn test_secret_restrictions_deserialization_invalid_country() {
        let json = r#"{"allowed_countries": ["invalid"]}"#;
        let result: std::result::Result<SecretRestrictions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Expected error for invalid country, got: {:?}",
            result
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("CountryCode must be a 2-letter uppercase ISO 3166-1 alpha-2 code")
        );
    }

    #[test]
    fn test_secret_restrictions_deserialization_invalid_asn() {
        let json = r#"{"allowed_asns": ["invalid"]}"#;
        let result: std::result::Result<SecretRestrictions, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Expected error for invalid ASN, got: {:?}",
            result
        );
    }

    #[test]
    fn test_format_display_ips() {
        use ipnet::IpNet;

        // Test with multiple IPs and CIDR ranges
        let restrictions = SecretRestrictions::default().with_allowed_ips(vec![
            "127.0.0.1/32".parse::<IpNet>().unwrap(),
            "192.168.1.0/24".parse::<IpNet>().unwrap(),
            "::1/128".parse::<IpNet>().unwrap(),
        ]);
        assert_eq!(
            restrictions.to_string(),
            "Allowed IPs: 127.0.0.1/32, 192.168.1.0/24, ::1/128"
        );
    }

    #[test]
    fn test_format_display_countries() {
        let restrictions = SecretRestrictions::default().with_allowed_countries(vec![
            CountryCode::new("US").unwrap(),
            CountryCode::new("DE").unwrap(),
            CountryCode::new("CA").unwrap(),
        ]);
        assert_eq!(restrictions.to_string(), "Allowed Countries: US, DE, CA");
    }

    #[test]
    fn test_format_display_asns() {
        let restrictions = SecretRestrictions::default().with_allowed_asns(vec![48821, 202739]);
        assert_eq!(restrictions.to_string(), "Allowed ASNs: 48821, 202739");
    }

    #[test]
    fn test_format_display_all() {
        use ipnet::IpNet;

        let restrictions = SecretRestrictions::default()
            .with_allowed_ips(vec!["192.168.1.0/24".parse::<IpNet>().unwrap()])
            .with_allowed_countries(vec![CountryCode::new("US").unwrap()])
            .with_allowed_asns(vec![202739]);

        let display = restrictions.to_string();
        assert!(display.contains("Allowed IPs: 192.168.1.0/24"));
        assert!(display.contains("Allowed Countries: US"));
        assert!(display.contains("Allowed ASNs: 202739"));
    }

    #[test]
    fn test_format_display_empty() {
        // Test with no restrictions
        let restrictions = SecretRestrictions::default();
        assert_eq!(restrictions.to_string(), "No restrictions");

        // Test with empty IP list
        let restrictions = SecretRestrictions::default();
        assert_eq!(restrictions.to_string(), "No restrictions");
    }

    #[test]
    fn test_with_allowed_countries() {
        let restrictions = SecretRestrictions::default().with_allowed_countries(vec![
            CountryCode::new("US").unwrap(),
            CountryCode::new("DE").unwrap(),
        ]);

        assert!(restrictions.allowed_ips.is_none());
        assert_eq!(restrictions.allowed_countries.unwrap().len(), 2);
    }

    #[test]
    fn test_default_is_empty() {
        let restrictions = SecretRestrictions::default();
        assert!(restrictions.is_empty());
    }

    #[test]
    fn test_is_with_empty_ips() {
        let restrictions = SecretRestrictions::default().with_allowed_ips(vec![]);
        assert!(restrictions.is_empty());
    }

    #[test]
    fn test_is_with_empty_countries() {
        let restrictions = SecretRestrictions::default().with_allowed_countries(vec![]);
        assert!(restrictions.is_empty());
    }

    #[test]
    fn test_is_with_empty_asns() {
        let restrictions = SecretRestrictions::default().with_allowed_asns(vec![]);
        assert!(restrictions.is_empty());
    }

    #[test]
    fn test_is_with_ips() {
        let ip = "127.0.0.1/32".parse::<IpNet>().unwrap();
        let restrictions = SecretRestrictions::default().with_allowed_ips(vec![ip]);
        assert!(!restrictions.is_empty());
    }

    #[test]
    fn test_is_with_countries() {
        let country = CountryCode::new("DE").unwrap();
        let restrictions = SecretRestrictions::default().with_allowed_countries(vec![country]);
        assert!(!restrictions.is_empty());
    }

    #[test]
    fn test_is_with_asns() {
        let restrictions = SecretRestrictions::default().with_allowed_asns(vec![202739]);
        assert!(!restrictions.is_empty());
    }

    // Tests for passphrase functionality
    #[test]
    fn test_with_passphrase_basic() {
        let restrictions = SecretRestrictions::default().with_passphrase(b"mypassword");

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Passphrase hash should be set"
        );
        assert!(
            !restrictions.is_empty(),
            "Restrictions with passphrase should not be empty"
        );
        // Should hash to a hex string (64 characters for SHA-256)
        let hash = restrictions.passphrase_hash.unwrap();
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 characters long");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should contain only hex digits"
        );
    }

    #[test]
    fn test_with_passphrase_empty() {
        let restrictions = SecretRestrictions::default().with_passphrase(b"");

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Empty passphrase should still produce a hash"
        );
        assert!(
            !restrictions.is_empty(),
            "Restrictions with empty passphrase should not be empty"
        );
        let hash = restrictions.passphrase_hash.unwrap();
        assert_eq!(
            hash.len(),
            64,
            "Empty passphrase hash should be 64 characters"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Empty passphrase hash should be valid hex"
        );
    }

    #[test]
    fn test_with_passphrase_unicode() {
        let unicode_phrase = "パスワード123🔒";
        let restrictions = SecretRestrictions::default().with_passphrase(unicode_phrase.as_bytes());

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Unicode passphrase should be hashed"
        );
        assert!(
            !restrictions.is_empty(),
            "Restrictions with unicode passphrase should not be empty"
        );
        let hash = restrictions.passphrase_hash.unwrap();
        assert_eq!(
            hash.len(),
            64,
            "Unicode passphrase hash should be 64 characters"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Unicode passphrase hash should be valid hex"
        );
    }

    #[test]
    fn test_with_passphrase_binary() {
        let binary_data = vec![0u8, 1, 2, 3, 255, 254, 253, 128, 127];
        let restrictions = SecretRestrictions::default().with_passphrase(&binary_data);

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Binary passphrase should be hashed"
        );
        assert!(
            !restrictions.is_empty(),
            "Restrictions with binary passphrase should not be empty"
        );
        let hash = restrictions.passphrase_hash.unwrap();
        assert_eq!(
            hash.len(),
            64,
            "Binary passphrase hash should be 64 characters"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Binary passphrase hash should be valid hex"
        );
    }

    #[test]
    fn test_with_passphrase_special_characters() {
        let special_chars = b"!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let restrictions = SecretRestrictions::default().with_passphrase(special_chars);

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Special character passphrase should be hashed"
        );
        assert!(
            !restrictions.is_empty(),
            "Restrictions with special char passphrase should not be empty"
        );
        let hash = restrictions.passphrase_hash.unwrap();
        assert_eq!(
            hash.len(),
            64,
            "Special char passphrase hash should be 64 characters"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Special char passphrase hash should be valid hex"
        );
    }

    #[test]
    fn test_with_passphrase_consistency() {
        let passphrase = b"consistent_test";
        let restrictions1 = SecretRestrictions::default().with_passphrase(passphrase);
        let restrictions2 = SecretRestrictions::default().with_passphrase(passphrase);

        assert_eq!(
            restrictions1.passphrase_hash, restrictions2.passphrase_hash,
            "Same passphrase should produce identical hashes"
        );
    }

    #[test]
    fn test_with_passphrase_different_inputs() {
        let restrictions1 = SecretRestrictions::default().with_passphrase(b"password1");
        let restrictions2 = SecretRestrictions::default().with_passphrase(b"password2");

        assert_ne!(
            restrictions1.passphrase_hash, restrictions2.passphrase_hash,
            "Different passphrases should produce different hashes"
        );
    }

    #[test]
    fn test_with_passphrase_case_sensitive() {
        let restrictions1 = SecretRestrictions::default().with_passphrase(b"Password");
        let restrictions2 = SecretRestrictions::default().with_passphrase(b"password");

        assert_ne!(
            restrictions1.passphrase_hash, restrictions2.passphrase_hash,
            "Case-different passphrases should produce different hashes"
        );
    }

    #[test]
    fn test_with_passphrase_whitespace_sensitive() {
        let restrictions1 = SecretRestrictions::default().with_passphrase(b"password");
        let restrictions2 = SecretRestrictions::default().with_passphrase(b" password");
        let restrictions3 = SecretRestrictions::default().with_passphrase(b"password ");

        assert_ne!(
            restrictions1.passphrase_hash, restrictions2.passphrase_hash,
            "Leading whitespace should change the hash"
        );
        assert_ne!(
            restrictions1.passphrase_hash, restrictions3.passphrase_hash,
            "Trailing whitespace should change the hash"
        );
        assert_ne!(
            restrictions2.passphrase_hash, restrictions3.passphrase_hash,
            "Leading vs trailing whitespace should produce different hashes"
        );
    }

    #[test]
    fn test_with_passphrase_long_input() {
        let long_passphrase = vec![b'a'; 1000];
        let restrictions = SecretRestrictions::default().with_passphrase(&long_passphrase);

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Long passphrase should be hashed"
        );
        assert!(
            !restrictions.is_empty(),
            "Restrictions with long passphrase should not be empty"
        );
        let hash = restrictions.passphrase_hash.unwrap();
        assert_eq!(
            hash.len(),
            64,
            "Long passphrase hash should be 64 characters"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Long passphrase hash should be valid hex"
        );
    }

    #[test]
    fn test_with_passphrase_combined_with_other_restrictions() {
        use ipnet::IpNet;

        let ip = "192.168.1.0/24".parse::<IpNet>().unwrap();
        let country = CountryCode::new("US").unwrap();
        let restrictions = SecretRestrictions::default()
            .with_allowed_ips(vec![ip])
            .with_allowed_countries(vec![country])
            .with_allowed_asns(vec![13335])
            .with_passphrase(b"comprehensive");

        assert!(
            restrictions.allowed_ips.is_some(),
            "IP restrictions should be set"
        );
        assert!(
            restrictions.allowed_countries.is_some(),
            "Country restrictions should be set"
        );
        assert!(
            restrictions.allowed_asns.is_some(),
            "ASN restrictions should be set"
        );
        assert!(
            restrictions.passphrase_hash.is_some(),
            "Passphrase hash should be set"
        );
        assert!(
            !restrictions.is_empty(),
            "Combined restrictions should not be empty"
        );
    }

    #[test]
    fn test_is_empty_with_passphrase() {
        let restrictions = SecretRestrictions::default().with_passphrase(b"test");
        assert!(
            !restrictions.is_empty(),
            "Restrictions with passphrase should not be empty"
        );
    }

    #[test]
    fn test_is_empty_with_empty_string_passphrase_hash() {
        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(String::new());
        assert!(
            restrictions.is_empty(),
            "Empty string passphrase hash should be considered empty"
        );
    }

    #[test]
    fn test_is_empty_with_whitespace_passphrase_hash() {
        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some("   ".to_string());
        assert!(
            !restrictions.is_empty(),
            "Whitespace passphrase hash should not be considered empty"
        );
    }

    #[test]
    fn test_display_with_passphrase_only() {
        let restrictions = SecretRestrictions::default().with_passphrase(b"secret");
        let display = restrictions.to_string();
        assert!(
            display.contains("Passphrase: ***"),
            "Display should show masked passphrase"
        );
        assert!(
            !display.contains("No restrictions"),
            "Display should not show 'No restrictions' when passphrase is set"
        );
    }

    #[test]
    fn test_display_with_passphrase_and_other_restrictions() {
        use ipnet::IpNet;

        let ip = "192.168.1.0/24".parse::<IpNet>().unwrap();
        let restrictions = SecretRestrictions::default()
            .with_allowed_ips(vec![ip])
            .with_passphrase(b"secret");

        let display = restrictions.to_string();
        assert!(
            display.contains("Allowed IPs: 192.168.1.0/24"),
            "Display should show IP restrictions"
        );
        assert!(
            display.contains("Passphrase: ***"),
            "Display should show masked passphrase"
        );
    }

    #[test]
    fn test_serialization_with_passphrase() {
        let restrictions = SecretRestrictions::default().with_passphrase(b"test");
        let serialized = serde_json::to_string(&restrictions).unwrap();

        assert!(
            serialized.contains("passphrase_hash"),
            "Serialized JSON should contain passphrase_hash field"
        );
        assert!(
            serialized.contains("\"passphrase_hash\":"),
            "Serialized JSON should have proper passphrase_hash structure"
        );

        // Should not contain the actual passphrase
        assert!(
            !serialized.contains("test"),
            "Serialized JSON should not contain plaintext passphrase"
        );
        // Should contain the hash (64 hex characters)
        assert!(
            serialized.len() > 80,
            "Serialized JSON should be reasonably long to contain the hash"
        ); // Account for JSON structure
    }

    #[test]
    fn test_deserialization_with_passphrase() {
        let json = r#"{"passphrase_hash": "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        assert!(
            restrictions.passphrase_hash.is_some(),
            "Deserialized restrictions should have passphrase hash"
        );
        assert_eq!(
            restrictions.passphrase_hash.as_ref().unwrap(),
            "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",
            "Deserialized passphrase hash should match expected value"
        );
        assert!(
            !restrictions.is_empty(),
            "Deserialized restrictions with passphrase should not be empty"
        );
    }

    #[test]
    fn test_deserialization_with_passphrase_and_other_fields() {
        let json = r#"{"allowed_ips": ["192.168.1.0/24"], "passphrase_hash": "abc123"}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        assert!(
            restrictions.allowed_ips.is_some(),
            "Deserialized restrictions should have allowed IPs"
        );
        assert!(
            restrictions.passphrase_hash.is_some(),
            "Deserialized restrictions should have passphrase hash"
        );
        assert_eq!(
            restrictions.passphrase_hash.unwrap(),
            "abc123",
            "Deserialized passphrase hash should match expected value"
        );
    }
}
