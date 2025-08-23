// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

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
}

impl SecretRestrictions {
    /// Creates a new SecretRestrictions with IP restrictions
    pub fn with_allowed_ips(allowed_ips: Vec<ipnet::IpNet>) -> Self {
        Self {
            allowed_ips: Some(allowed_ips),
        }
    }

    /// Checks if any restrictions are set
    pub fn is_empty(&self) -> bool {
        self.allowed_ips.is_none()
    }

    /// Pretty format restrictions for display in logs, webhooks, etc.
    pub fn format_display(&self) -> Option<String> {
        if let Some(ips) = &self.allowed_ips {
            if ips.is_empty() {
                return None;
            }

            let ip_strings: Vec<String> = ips.iter().map(|ip| ip.to_string()).collect();
            Some(format!("IP Restrictions: {}", ip_strings.join(", ")))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_secret_restrictions_deserialization_empty() {
        // Test with null allowed_ips
        let json = r#"{"allowed_ips": null}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());

        // Test with empty object
        let json = r#"{}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());
    }

    #[test]
    fn test_secret_restrictions_deserialization_invalid_ip() {
        let json = r#"{"allowed_ips": ["invalid-ip"]}"#;
        let result: std::result::Result<SecretRestrictions, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid IP address or CIDR notation")
        );
    }

    #[test]
    fn test_format_display() {
        use ipnet::IpNet;

        // Test with multiple IPs and CIDR ranges
        let restrictions = SecretRestrictions::with_allowed_ips(vec![
            "127.0.0.1/32".parse::<IpNet>().unwrap(),
            "192.168.1.0/24".parse::<IpNet>().unwrap(),
            "::1/128".parse::<IpNet>().unwrap(),
        ]);
        let formatted = restrictions.format_display().unwrap();
        assert_eq!(
            formatted,
            "IP Restrictions: 127.0.0.1/32, 192.168.1.0/24, ::1/128"
        );
    }

    #[test]
    fn test_format_display_empty() {
        // Test with no restrictions
        let restrictions = SecretRestrictions::default();
        assert!(restrictions.format_display().is_none());

        // Test with empty IP list
        let restrictions = SecretRestrictions {
            allowed_ips: Some(vec![]),
        };
        assert!(restrictions.format_display().is_none());
    }

    #[test]
    fn test_format_display_single_ip() {
        use ipnet::IpNet;

        let restrictions =
            SecretRestrictions::with_allowed_ips(vec!["10.0.0.1/32".parse::<IpNet>().unwrap()]);
        let formatted = restrictions.format_display().unwrap();
        assert_eq!(formatted, "IP Restrictions: 10.0.0.1/32");
    }
}
