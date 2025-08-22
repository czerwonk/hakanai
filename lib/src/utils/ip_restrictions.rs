// SPDX-License-Identifier: Apache-2.0

//! IP restriction utilities for access control.
//!
//! This module provides utilities for checking IP addresses against allowed ranges,
//! supporting both IPv4 and IPv6 networks with CIDR notation.

use std::net::IpAddr;

/// Checks if an IP address is allowed based on a list of permitted networks.
///
/// # Arguments
///
/// * `client_ip` - The IP address to check
/// * `allowed_networks` - Optional list of permitted IP networks. If None, all IPs are allowed.
///
/// # Returns
///
/// * `true` - If the IP is allowed (either no restrictions or IP matches a range)
/// * `false` - If restrictions exist and the IP doesn't match any allowed range
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
/// use hakanai_lib::utils::ip_restrictions::is_ip_allowed;
///
/// // No restrictions - all IPs allowed
/// assert!(is_ip_allowed(&"192.168.1.100".parse().unwrap(), None));
///
/// // With restrictions
/// let allowed = vec![
///     "192.168.1.0/24".parse().unwrap(),
///     "10.0.0.0/8".parse().unwrap(),
/// ];
///
/// assert!(is_ip_allowed(&"192.168.1.50".parse().unwrap(), Some(&allowed)));
/// assert!(!is_ip_allowed(&"172.16.1.1".parse().unwrap(), Some(&allowed)));
/// ```
pub fn is_ip_allowed(client_ip: &IpAddr, allowed_networks: Option<&[ipnet::IpNet]>) -> bool {
    match allowed_networks {
        Some(networks) => {
            // If restrictions exist, check if the IP is in any of the allowed ranges
            networks.iter().any(|range| range.contains(client_ip))
        }
        None => {
            // No restrictions means all IPs are allowed
            true
        }
    }
}

/// Serializes IP networks to JSON string for storage.
///
/// # Arguments
///
/// * `networks` - Vector of IP networks to serialize
///
/// # Returns
///
/// * `Ok(String)` - JSON representation of the networks
/// * `Err(String)` - Serialization error message
pub fn serialize_ip_networks(networks: &[ipnet::IpNet]) -> Result<String, String> {
    serde_json::to_string(networks)
        .map_err(|e| format!("Failed to serialize IP restrictions: {}", e))
}

/// Deserializes IP networks from JSON string.
///
/// # Arguments
///
/// * `json` - JSON string containing serialized IP networks
///
/// # Returns
///
/// * `Ok(Vec<ipnet::IpNet>)` - Parsed IP networks
/// * `Err(String)` - Deserialization error message
pub fn deserialize_ip_networks(json: &str) -> Result<Vec<ipnet::IpNet>, String> {
    serde_json::from_str(json).map_err(|e| format!("Failed to deserialize IP restrictions: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test IP networks
    fn test_ip_networks() -> Vec<ipnet::IpNet> {
        vec![
            "192.168.1.0/24".parse().unwrap(),
            "10.0.0.100/32".parse().unwrap(),
            "2001:db8::/32".parse().unwrap(),
        ]
    }

    #[test]
    fn test_no_restrictions_allows_all() {
        // No restrictions should allow any IP
        assert!(is_ip_allowed(&"192.168.1.1".parse().unwrap(), None));
        assert!(is_ip_allowed(&"10.0.0.1".parse().unwrap(), None));
        assert!(is_ip_allowed(&"::1".parse().unwrap(), None));
        assert!(is_ip_allowed(&"8.8.8.8".parse().unwrap(), None));
    }

    #[test]
    fn test_ipv4_network_restrictions() {
        let networks = vec!["192.168.1.0/24".parse().unwrap()];

        // IPs within range should be allowed
        assert!(is_ip_allowed(
            &"192.168.1.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(is_ip_allowed(
            &"192.168.1.100".parse().unwrap(),
            Some(&networks)
        ));
        assert!(is_ip_allowed(
            &"192.168.1.254".parse().unwrap(),
            Some(&networks)
        ));

        // IPs outside range should be denied
        assert!(!is_ip_allowed(
            &"192.168.2.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(
            &"10.0.0.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(&"8.8.8.8".parse().unwrap(), Some(&networks)));
    }

    #[test]
    fn test_ipv6_network_restrictions() {
        let networks = vec!["2001:db8::/32".parse().unwrap()];

        // IPv6 addresses within range should be allowed
        assert!(is_ip_allowed(
            &"2001:db8::1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(is_ip_allowed(
            &"2001:db8:1234::1".parse().unwrap(),
            Some(&networks)
        ));

        // IPv6 addresses outside range should be denied
        assert!(!is_ip_allowed(
            &"2001:db9::1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(&"::1".parse().unwrap(), Some(&networks)));

        // IPv4 addresses should be denied when only IPv6 ranges are allowed
        assert!(!is_ip_allowed(
            &"192.168.1.1".parse().unwrap(),
            Some(&networks)
        ));
    }

    #[test]
    fn test_single_ip_restrictions() {
        let networks = vec!["192.168.1.100/32".parse().unwrap()];

        // Exact IP match should be allowed
        assert!(is_ip_allowed(
            &"192.168.1.100".parse().unwrap(),
            Some(&networks)
        ));

        // Any other IP should be denied
        assert!(!is_ip_allowed(
            &"192.168.1.101".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(
            &"192.168.1.99".parse().unwrap(),
            Some(&networks)
        ));
    }

    #[test]
    fn test_mixed_ipv4_ipv6_restrictions() {
        let networks = vec![
            "192.168.1.0/24".parse().unwrap(),
            "10.0.0.100/32".parse().unwrap(),
            "2001:db8::/32".parse().unwrap(),
            "::1/128".parse().unwrap(),
        ];

        // IPv4 addresses in allowed ranges
        assert!(is_ip_allowed(
            &"192.168.1.50".parse().unwrap(),
            Some(&networks)
        ));
        assert!(is_ip_allowed(
            &"10.0.0.100".parse().unwrap(),
            Some(&networks)
        ));

        // IPv6 addresses in allowed ranges
        assert!(is_ip_allowed(
            &"2001:db8::1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(is_ip_allowed(&"::1".parse().unwrap(), Some(&networks)));

        // Addresses outside any range should be denied
        assert!(!is_ip_allowed(
            &"172.16.1.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(
            &"2001:db9::1".parse().unwrap(),
            Some(&networks)
        ));
    }

    #[test]
    fn test_empty_restrictions() {
        let networks: Vec<ipnet::IpNet> = vec![];

        // Empty restrictions should deny all access
        assert!(!is_ip_allowed(
            &"192.168.1.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(&"::1".parse().unwrap(), Some(&networks)));
    }

    #[test]
    fn test_localhost_restrictions() {
        let networks = vec!["127.0.0.0/8".parse().unwrap(), "::1/128".parse().unwrap()];

        // IPv4 localhost
        assert!(is_ip_allowed(
            &"127.0.0.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(is_ip_allowed(
            &"127.1.2.3".parse().unwrap(),
            Some(&networks)
        ));

        // IPv6 localhost
        assert!(is_ip_allowed(&"::1".parse().unwrap(), Some(&networks)));

        // Non-localhost should be denied
        assert!(!is_ip_allowed(
            &"192.168.1.1".parse().unwrap(),
            Some(&networks)
        ));
        assert!(!is_ip_allowed(
            &"2001:db8::1".parse().unwrap(),
            Some(&networks)
        ));
    }

    #[test]
    fn test_serialize_deserialize_ip_networks() {
        let networks = test_ip_networks();

        // Test serialization
        let json = serialize_ip_networks(&networks).unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("192.168.1.0/24"));
        assert!(json.contains("2001:db8::/32"));

        // Test deserialization
        let deserialized = deserialize_ip_networks(&json).unwrap();
        assert_eq!(networks.len(), deserialized.len());

        for (original, parsed) in networks.iter().zip(deserialized.iter()) {
            assert_eq!(original, parsed);
        }
    }

    #[test]
    fn test_serialize_empty_networks() {
        let empty_networks: Vec<ipnet::IpNet> = vec![];
        let json = serialize_ip_networks(&empty_networks).unwrap();
        assert_eq!(json, "[]");

        let deserialized = deserialize_ip_networks(&json).unwrap();
        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let invalid_json = "{ invalid json }";
        let result = deserialize_ip_networks(invalid_json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Failed to deserialize IP restrictions")
        );
    }

    #[test]
    fn test_deserialize_wrong_format() {
        let wrong_format = r#"["not-an-ip", "192.168.1.0/24"]"#;
        let result = deserialize_ip_networks(wrong_format);
        assert!(result.is_err());
    }

    #[test]
    fn test_private_network_ranges() {
        let private_networks = vec![
            "10.0.0.0/8".parse().unwrap(),
            "172.16.0.0/12".parse().unwrap(),
            "192.168.0.0/16".parse().unwrap(),
        ];

        // Test various private IP addresses
        assert!(is_ip_allowed(
            &"10.1.2.3".parse().unwrap(),
            Some(&private_networks)
        ));
        assert!(is_ip_allowed(
            &"172.16.1.1".parse().unwrap(),
            Some(&private_networks)
        ));
        assert!(is_ip_allowed(
            &"172.31.255.255".parse().unwrap(),
            Some(&private_networks)
        ));
        assert!(is_ip_allowed(
            &"192.168.100.200".parse().unwrap(),
            Some(&private_networks)
        ));

        // Test public IP addresses (should be denied)
        assert!(!is_ip_allowed(
            &"8.8.8.8".parse().unwrap(),
            Some(&private_networks)
        ));
        assert!(!is_ip_allowed(
            &"1.1.1.1".parse().unwrap(),
            Some(&private_networks)
        ));
        assert!(!is_ip_allowed(
            &"172.15.1.1".parse().unwrap(),
            Some(&private_networks)
        )); // Just outside 172.16.0.0/12
    }
}
