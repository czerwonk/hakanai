// SPDX-License-Identifier: Apache-2.0

//! IP address and CIDR notation parsing utilities.
//!
//! This module provides utilities for parsing IP addresses and CIDR notation,
//! supporting both IPv4 and IPv6 addresses. Single IP addresses are automatically
//! converted to /32 (IPv4) or /128 (IPv6) CIDR ranges.

use std::net::IpAddr;

use serde::{Deserialize, Deserializer};

/// Parses an IP network from a string (supports both single IPs and CIDR notation).
///
/// This function first attempts to parse the input as CIDR notation. If that fails,
/// it tries to parse as a single IP address and converts it to a host route
/// (/32 for IPv4, /128 for IPv6).
///
/// # Arguments
///
/// * `s` - A string slice that should contain an IP address or CIDR notation
///
/// # Returns
///
/// * `Ok(IpNet)` - Successfully parsed IP network
/// * `Err(String)` - Error message describing the parsing failure
///
/// # Examples
///
/// ```
/// use hakanai_lib::utils::ip::parse_ipnet;
///
/// // CIDR notation
/// let network = parse_ipnet("192.168.1.0/24").unwrap();
/// assert_eq!(network.to_string(), "192.168.1.0/24");
///
/// // Single IPv4 address (converted to /32)
/// let host = parse_ipnet("192.168.1.100").unwrap();
/// assert_eq!(host.to_string(), "192.168.1.100/32");
///
/// // Single IPv6 address (converted to /128)
/// let ipv6_host = parse_ipnet("2001:db8::1").unwrap();
/// assert_eq!(ipv6_host.to_string(), "2001:db8::1/128");
///
/// // Invalid input
/// assert!(parse_ipnet("not-an-ip").is_err());
/// ```
pub fn parse_ipnet(s: &str) -> Result<ipnet::IpNet, String> {
    // Try to parse as CIDR first
    if let Ok(ipnet) = s.parse::<ipnet::IpNet>() {
        return Ok(ipnet);
    }

    // If that fails, try to parse as a single IP and convert to CIDR
    if let Ok(ip) = s.parse::<IpAddr>() {
        let ipnet = match ip {
            IpAddr::V4(ipv4) => ipnet::Ipv4Net::new(ipv4, 32)
                .map(ipnet::IpNet::V4)
                .map_err(|e| format!("Failed to create IPv4 network: {}", e))?,
            IpAddr::V6(ipv6) => ipnet::Ipv6Net::new(ipv6, 128)
                .map(ipnet::IpNet::V6)
                .map_err(|e| format!("Failed to create IPv6 network: {}", e))?,
        };
        return Ok(ipnet);
    }

    Err(format!("Invalid IP address or CIDR notation: {}", s))
}

/// Custom deserializer for converting JSON string arrays to Vec<ipnet::IpNet>
pub fn deserialize_ip_nets<'de, D>(deserializer: D) -> Result<Option<Vec<ipnet::IpNet>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // Handle both Vec<String> and null/missing cases
    let strings_opt = Option::<Vec<String>>::deserialize(deserializer)?;

    match strings_opt {
        Some(strings) => {
            let mut ip_nets = Vec::new();
            for s in strings {
                let ip_net = parse_ipnet(&s).map_err(Error::custom)?;
                ip_nets.push(ip_net);
            }
            Ok(Some(ip_nets))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4_single_address() {
        let result = parse_ipnet("192.168.1.100").unwrap();
        assert_eq!(result.to_string(), "192.168.1.100/32");
    }

    #[test]
    fn test_parse_ipv4_cidr() {
        let result = parse_ipnet("10.0.0.0/8").unwrap();
        assert_eq!(result.to_string(), "10.0.0.0/8");
    }

    #[test]
    fn test_parse_ipv6_single_address() {
        let result = parse_ipnet("2001:db8::1").unwrap();
        assert_eq!(result.to_string(), "2001:db8::1/128");
    }

    #[test]
    fn test_parse_ipv6_cidr() {
        let result = parse_ipnet("2001:db8::/32").unwrap();
        assert_eq!(result.to_string(), "2001:db8::/32");
    }

    #[test]
    fn test_parse_localhost_ipv4() {
        let result = parse_ipnet("127.0.0.1").unwrap();
        assert_eq!(result.to_string(), "127.0.0.1/32");
    }

    #[test]
    fn test_parse_localhost_ipv6() {
        let result = parse_ipnet("::1").unwrap();
        assert_eq!(result.to_string(), "::1/128");
    }

    #[test]
    fn test_parse_private_networks() {
        // Test common private network ranges
        let cases = vec![
            ("192.168.1.0/24", "192.168.1.0/24"),
            ("10.0.0.0/8", "10.0.0.0/8"),
            ("172.16.0.0/12", "172.16.0.0/12"),
        ];

        for (input, expected) in cases {
            let result = parse_ipnet(input).unwrap();
            assert_eq!(result.to_string(), expected);
        }
    }

    #[test]
    fn test_parse_invalid_ip() {
        let result = parse_ipnet("not-an-ip");
        assert!(
            result.is_err(),
            "Expected error for invalid IP, got: {:?}",
            result
        );
        assert!(
            result
                .unwrap_err()
                .contains("Invalid IP address or CIDR notation")
        );
    }

    #[test]
    fn test_parse_invalid_cidr() {
        // Invalid CIDR range for IPv4
        let result = parse_ipnet("192.168.1.0/33");
        assert!(
            result.is_err(),
            "Expected error for invalid CIDR, got: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_empty_string() {
        let result = parse_ipnet("");
        assert!(
            result.is_err(),
            "Expected error for empty string, got: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_malformed_addresses() {
        let invalid_cases = vec![
            "192.168.1",      // Incomplete IPv4
            "192.168.1.256",  // Invalid IPv4 octet
            "2001:db8::gggg", // Invalid IPv6 hex
            "192.168.1.0/",   // Missing CIDR value
            "192.168.1.0/-1", // Negative CIDR
        ];

        for invalid in invalid_cases {
            let result = parse_ipnet(invalid);
            assert!(result.is_err(), "Expected error for input: {}", invalid);
        }
    }

    #[test]
    fn test_parse_edge_case_cidrs() {
        // Test edge cases for CIDR ranges
        let cases = vec![
            ("0.0.0.0/0", "0.0.0.0/0"),           // All IPv4
            ("::/0", "::/0"),                     // All IPv6
            ("192.168.1.0/32", "192.168.1.0/32"), // Single host as CIDR
        ];

        for (input, expected) in cases {
            let result = parse_ipnet(input).unwrap();
            assert_eq!(result.to_string(), expected);
        }
    }
}
