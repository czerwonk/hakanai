// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::CountryCode;

/// Represents access restrictions for a secret.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SecretRestrictions {
    /// IP addresses/ranges allowed to access the secret
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_utils::deserialize_ip_nets"
    )]
    pub allowed_ips: Option<Vec<ipnet::IpNet>>,

    /// Geo-IP locations allowed to access the secret
    pub allowed_countries: Option<Vec<CountryCode>>,

    /// ASNs allowed to access the secret
    pub allowed_asns: Option<Vec<u32>>,
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
}
