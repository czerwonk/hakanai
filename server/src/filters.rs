// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;
use std::str::FromStr;

use actix_web::HttpRequest;

use hakanai_lib::models::CountryCode;

use crate::app_data::AppData;

/// Check if the request is from a whitelisted IP range
pub fn is_request_from_whitelisted_ip(req: &HttpRequest, app_data: &AppData) -> bool {
    if let Some(ref trusted_ranges) = app_data.trusted_ip_ranges {
        return is_request_from_ip_range(req, app_data, trusted_ranges);
    };

    false
}

/// Checks if the request is from one of the given IP ranges
pub fn is_request_from_ip_range(
    req: &HttpRequest,
    app_data: &AppData,
    ranges: &[ipnet::IpNet],
) -> bool {
    if let Some(client_ip) = extract_client_ip(req, &app_data.trusted_ip_header)
        && is_ip_in_ranges(&client_ip, ranges)
    {
        return true;
    }

    false
}

/// Checks if the request is from one of the given countries
pub fn is_request_from_country(
    req: &HttpRequest,
    app_data: &AppData,
    countries: &[CountryCode],
) -> bool {
    let header_name = &app_data.country_header;

    if let Some(name) = header_name
        && let Some(header_value) = extract_header_value(req, name)
    {
        return countries
            .iter()
            .any(|c| c.as_str() == header_value.to_uppercase());
    }

    false
}

/// Checks if the request is from one of the given autonomous systems
pub fn is_request_from_asn(req: &HttpRequest, app_data: &AppData, asns: &[u32]) -> bool {
    let header_name = &app_data.asn_header;

    if let Some(name) = header_name
        && let Some(header_value) = extract_header_value(req, name)
        && let Ok(asn) = header_value.parse::<u32>()
    {
        return asns.contains(&asn);
    }

    false
}

/// Extract client IP from request headers or connection info
fn extract_client_ip(req: &HttpRequest, trusted_header: &str) -> Option<IpAddr> {
    // First check the configured trusted header (e.g., x-forwarded-for)
    if let Some(header_value) = extract_header_value(req, trusted_header) {
        // Handle comma-separated IPs (take the first one)
        if let Some(first_ip) = header_value.split(',').next()
            && let Ok(ip) = IpAddr::from_str(first_ip.trim())
        {
            return Some(ip);
        }
    }

    // Fallback to connection peer address
    req.connection_info()
        .peer_addr()
        .and_then(|addr| addr.split(':').next()) // Remove port
        .and_then(|ip_str| IpAddr::from_str(ip_str).ok())
}

/// Extract and trim header value value from the request
pub fn extract_header_value(req: &HttpRequest, header_name: &str) -> Option<String> {
    req.headers()
        .get(header_name)
        .and_then(|h| h.to_str().ok().map(|s| s.trim().to_string()))
}

/// Check if an IP address is in any of the provided ranges
fn is_ip_in_ranges(ip: &IpAddr, ranges: &[ipnet::IpNet]) -> bool {
    ranges.iter().any(|range| range.contains(ip))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_data::{AnonymousOptions, AppData};
    use actix_web::{HttpRequest, test};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::time::Duration;

    fn create_test_app_data(trusted_ranges: Option<Vec<ipnet::IpNet>>, header: &str) -> AppData {
        AppData::default()
            .with_max_ttl(Duration::from_secs(7200))
            .with_anonymous_usage(AnonymousOptions {
                allowed: true,
                upload_size_limit: 32 * 1024,
            })
            .with_trusted_ip_ranges(trusted_ranges)
            .with_trusted_ip_header(header.to_string())
    }

    fn create_request_with_headers(headers: &[(&str, &str)]) -> HttpRequest {
        let mut req = test::TestRequest::get().uri("/");
        for (name, value) in headers {
            req = req.insert_header((*name, *value));
        }
        req.to_http_request()
    }

    #[actix_web::test]
    async fn test_is_ip_in_ranges_ipv4() {
        let ranges = vec![
            "10.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
            "192.168.1.0/24".parse::<ipnet::IpNet>().unwrap(),
        ];

        // Test IPs in range
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        assert!(is_ip_in_ranges(&ip, &ranges));

        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        assert!(is_ip_in_ranges(&ip, &ranges));

        // Test IPs not in range
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        assert!(!is_ip_in_ranges(&ip, &ranges));

        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1));
        assert!(!is_ip_in_ranges(&ip, &ranges));
    }

    #[actix_web::test]
    async fn test_is_ip_in_ranges_ipv6() {
        let ranges = vec![
            "2001:db8::/32".parse::<ipnet::IpNet>().unwrap(),
            "::1/128".parse::<ipnet::IpNet>().unwrap(),
        ];

        // Test IPs in range
        let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        assert!(is_ip_in_ranges(&ip, &ranges));

        let ip = IpAddr::V6(Ipv6Addr::LOCALHOST);
        assert!(is_ip_in_ranges(&ip, &ranges));

        // Test IPs not in range
        let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb9, 0, 0, 0, 0, 0, 1));
        assert!(!is_ip_in_ranges(&ip, &ranges));

        let ip = IpAddr::V6(Ipv6Addr::new(0x2002, 0, 0, 0, 0, 0, 0, 1));
        assert!(!is_ip_in_ranges(&ip, &ranges));
    }

    #[actix_web::test]
    async fn test_is_ip_in_ranges_mixed() {
        let ranges = vec![
            "10.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
            "2001:db8::/32".parse::<ipnet::IpNet>().unwrap(),
        ];

        // Test IPv4 in range
        let ip = IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3));
        assert!(is_ip_in_ranges(&ip, &ranges));

        // Test IPv6 in range
        let ip = IpAddr::V6(Ipv6Addr::new(
            0x2001, 0xdb8, 0x85a3, 0, 0, 0x8a2e, 0x370, 0x7334,
        ));
        assert!(is_ip_in_ranges(&ip, &ranges));

        // Test IPv4 not in range
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        assert!(!is_ip_in_ranges(&ip, &ranges));

        // Test IPv6 not in range
        let ip = IpAddr::V6(Ipv6Addr::LOCALHOST);
        assert!(!is_ip_in_ranges(&ip, &ranges));
    }

    #[actix_web::test]
    async fn test_is_ip_in_ranges_empty() {
        let ranges = vec![];
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        assert!(!is_ip_in_ranges(&ip, &ranges));
    }

    #[actix_web::test]
    async fn test_extract_header_value() {
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.1")]);
        let header = extract_header_value(&req, "x-forwarded-for");
        assert_eq!(header, Some("192.168.1.1".to_string()));

        let header = extract_header_value(&req, "nonexistent-header");
        assert_eq!(header, None);
    }

    #[actix_web::test]
    async fn test_extract_header_value_with_whitespace() {
        let req = create_request_with_headers(&[("x-forwarded-for", "  192.168.1.1  ")]);
        let header = extract_header_value(&req, "x-forwarded-for");
        assert_eq!(header, Some("192.168.1.1".to_string()));
    }

    #[actix_web::test]
    async fn test_extract_client_ip_from_header() {
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.100")]);
        let ip = extract_client_ip(&req, "x-forwarded-for");
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))));
    }

    #[actix_web::test]
    async fn test_extract_client_ip_from_header_multiple_ips() {
        let req = create_request_with_headers(&[(
            "x-forwarded-for",
            "192.168.1.100, 10.0.0.1, 172.16.0.1",
        )]);
        let ip = extract_client_ip(&req, "x-forwarded-for");
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))));
    }

    #[actix_web::test]
    async fn test_extract_client_ip_from_header_ipv6() {
        let req = create_request_with_headers(&[("x-forwarded-for", "2001:db8::1")]);
        let ip = extract_client_ip(&req, "x-forwarded-for");
        assert_eq!(
            ip,
            Some(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)))
        );
    }

    #[actix_web::test]
    async fn test_extract_client_ip_invalid_header() {
        let req = create_request_with_headers(&[("x-forwarded-for", "invalid-ip")]);
        let ip = extract_client_ip(&req, "x-forwarded-for");
        assert_eq!(ip, None);
    }

    #[actix_web::test]
    async fn test_extract_client_ip_missing_header() {
        let req = create_request_with_headers(&[]);
        let ip = extract_client_ip(&req, "x-forwarded-for");
        assert_eq!(ip, None);
    }

    #[actix_web::test]
    async fn test_extract_client_ip_cloudflare_header() {
        let req = create_request_with_headers(&[("cf-connecting-ip", "203.0.113.1")]);
        let ip = extract_client_ip(&req, "cf-connecting-ip");
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))));
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_true() {
        let ranges = vec![
            "10.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
            "2001:db8::/32".parse::<ipnet::IpNet>().unwrap(),
        ];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");
        let req = create_request_with_headers(&[("x-forwarded-for", "10.0.0.1")]);

        assert!(is_request_from_whitelisted_ip(&req, &app_data));
    }

    #[actix_web::test]
    async fn test_is_request_from_outside_whitelisted_ip_range() {
        let ranges = vec!["10.0.0.0/8".parse::<ipnet::IpNet>().unwrap()];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.1")]);

        assert!(
            !is_request_from_whitelisted_ip(&req, &app_data),
            "Should reject non-whitelisted IP"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_no_ranges() {
        let app_data = create_test_app_data(None, "x-forwarded-for");
        let req = create_request_with_headers(&[("x-forwarded-for", "10.0.0.1")]);

        assert!(
            !is_request_from_whitelisted_ip(&req, &app_data),
            "Should reject when no trusted IP ranges are configured"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_no_client_ip() {
        let ranges = vec!["10.0.0.0/8".parse::<ipnet::IpNet>().unwrap()];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");
        let req = create_request_with_headers(&[]);

        assert!(
            !is_request_from_whitelisted_ip(&req, &app_data),
            "Should reject when no client IP header is present"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_no_header_config() {
        let asns = vec![13335];
        let app_data = AppData::default().with_asn_header(None);

        let req = create_request_with_headers(&[("x-asn", "13335")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject when no ASN header is configured in AppData"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_missing_header() {
        let asns = vec![13335];
        let app_data = AppData::default().with_asn_header(Some("x-asn".to_string()));

        let req = create_request_with_headers(&[("x-different-header", "13335")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject when configured header 'x-asn' is missing but different header exists"
        );

        let req = create_request_with_headers(&[]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject when no headers are present"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_invalid_header_value() {
        let asns = vec![13335];
        let app_data = AppData::default().with_asn_header(Some("x-asn".to_string()));

        // Test non-numeric ASN
        let req = create_request_with_headers(&[("x-asn", "invalid-asn")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject non-numeric ASN value 'invalid-asn'"
        );

        // Test negative ASN (should fail parsing)
        let req = create_request_with_headers(&[("x-asn", "-1")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject negative ASN value '-1'"
        );

        // Test empty ASN
        let req = create_request_with_headers(&[("x-asn", "")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject empty ASN value"
        );

        // Test ASN with whitespace
        let req = create_request_with_headers(&[("x-asn", "  13335  ")]);
        assert!(
            is_request_from_asn(&req, &app_data, &asns),
            "Should accept ASN with whitespace after trimming"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_edge_case_values() {
        let app_data = AppData::default().with_asn_header(Some("x-asn".to_string()));

        // Test ASN 0 (reserved)
        let asns = vec![0];
        let req = create_request_with_headers(&[("x-asn", "0")]);
        assert!(
            is_request_from_asn(&req, &app_data, &asns),
            "Should accept reserved ASN 0"
        );

        // Test maximum 32-bit ASN (4294967295)
        let asns = vec![4294967295];
        let req = create_request_with_headers(&[("x-asn", "4294967295")]);
        assert!(
            is_request_from_asn(&req, &app_data, &asns),
            "Should accept maximum 32-bit ASN 4294967295"
        );

        // Test overflow (should fail parsing)
        let req = create_request_with_headers(&[("x-asn", "4294967296")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject ASN value that overflows u32 (4294967296)"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_multiple_asns() {
        let asns = vec![13335, 15169, 32934, 16509];
        let app_data = AppData::default().with_asn_header(Some("x-asn".to_string()));

        // Test each ASN individually
        for asn in &asns {
            let req = create_request_with_headers(&[("x-asn", &asn.to_string())]);
            assert!(
                is_request_from_asn(&req, &app_data, &asns),
                "Should accept allowed ASN {}",
                asn
            );
        }

        // Test non-allowed ASN
        let req = create_request_with_headers(&[("x-asn", "12345")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject non-allowed ASN 12345"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_empty_asn_list() {
        let asns = vec![];
        let app_data = AppData::default().with_asn_header(Some("x-asn".to_string()));

        let req = create_request_with_headers(&[("x-asn", "13335")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject when allowed ASN list is empty"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_different_headers() {
        let asns = vec![13335];

        // Test with custom ASN header
        let app_data = AppData::default().with_asn_header(Some("cf-asn".to_string()));
        let req = create_request_with_headers(&[("cf-asn", "13335"), ("x-asn", "15169")]);
        assert!(
            is_request_from_asn(&req, &app_data, &asns),
            "Should accept request when configured 'cf-asn' header matches"
        );

        // Test with different custom header
        let app_data = AppData::default().with_asn_header(Some("x-custom-asn".to_string()));
        let req = create_request_with_headers(&[("cf-asn", "13335"), ("x-custom-asn", "13335")]);
        assert!(
            is_request_from_asn(&req, &app_data, &asns),
            "Should accept request when configured 'x-custom-asn' header matches"
        );

        // Test when configured header doesn't match
        let app_data = AppData::default().with_asn_header(Some("x-missing-header".to_string()));
        let req = create_request_with_headers(&[("cf-asn", "13335"), ("x-asn", "13335")]);
        assert!(
            !is_request_from_asn(&req, &app_data, &asns),
            "Should reject when configured header 'x-missing-header' is not present"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_asn_real_world_asns() {
        let app_data = AppData::default().with_asn_header(Some("x-asn".to_string()));

        // Common real-world ASNs
        let cloudflare_asns = vec![13335];
        let req = create_request_with_headers(&[("x-asn", "13335")]);
        assert!(
            is_request_from_asn(&req, &app_data, &cloudflare_asns),
            "Should accept Cloudflare ASN 13335"
        );

        let google_asns = vec![15169];
        let req = create_request_with_headers(&[("x-asn", "15169")]);
        assert!(
            is_request_from_asn(&req, &app_data, &google_asns),
            "Should accept Google ASN 15169"
        );

        let amazon_asns = vec![16509];
        let req = create_request_with_headers(&[("x-asn", "16509")]);
        assert!(
            is_request_from_asn(&req, &app_data, &amazon_asns),
            "Should accept Amazon ASN 16509"
        );

        let microsoft_asns = vec![8075];
        let req = create_request_with_headers(&[("x-asn", "8075")]);
        assert!(
            is_request_from_asn(&req, &app_data, &microsoft_asns),
            "Should accept Microsoft ASN 8075"
        );
    }

    // Country-related tests
    #[actix_web::test]
    async fn test_is_request_from_country_basic() {
        use hakanai_lib::models::CountryCode;

        let countries = vec![
            CountryCode::from_str("US").unwrap(),
            CountryCode::from_str("DE").unwrap(),
        ];
        let app_data = AppData::default().with_country_header(Some("x-country".to_string()));

        // Test request from allowed country (lowercase)
        let req = create_request_with_headers(&[("x-country", "us")]);
        assert!(
            is_request_from_country(&req, &app_data, &countries),
            "Should accept request from US (lowercase 'us')"
        );

        // Test request from allowed country (uppercase)
        let req = create_request_with_headers(&[("x-country", "US")]);
        assert!(
            is_request_from_country(&req, &app_data, &countries),
            "Should accept request from US (uppercase 'US')"
        );

        // Test request from second allowed country
        let req = create_request_with_headers(&[("x-country", "de")]);
        assert!(
            is_request_from_country(&req, &app_data, &countries),
            "Should accept request from DE (lowercase 'de')"
        );

        // Test request from non-allowed country
        let req = create_request_with_headers(&[("x-country", "FR")]);
        assert!(
            !is_request_from_country(&req, &app_data, &countries),
            "Should reject request from non-allowed country FR"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_country_no_header_config() {
        use hakanai_lib::models::CountryCode;

        let countries = vec![CountryCode::from_str("US").unwrap()];
        let app_data = AppData::default().with_country_header(None);

        let req = create_request_with_headers(&[("x-country", "US")]);
        assert!(
            !is_request_from_country(&req, &app_data, &countries),
            "Should reject when no country header is configured in AppData"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_country_missing_header() {
        use hakanai_lib::models::CountryCode;

        let countries = vec![CountryCode::from_str("US").unwrap()];
        let app_data = AppData::default().with_country_header(Some("x-country".to_string()));

        let req = create_request_with_headers(&[("x-different-header", "US")]);
        assert!(
            !is_request_from_country(&req, &app_data, &countries),
            "Should reject when configured header 'x-country' is missing but different header exists"
        );

        let req = create_request_with_headers(&[]);
        assert!(
            !is_request_from_country(&req, &app_data, &countries),
            "Should reject when no headers are present"
        );
    }

    #[actix_web::test]
    async fn test_is_request_from_country_case_insensitive() {
        use hakanai_lib::models::CountryCode;

        let countries = vec![CountryCode::from_str("US").unwrap()];
        let app_data = AppData::default().with_country_header(Some("x-country".to_string()));

        // Test various case combinations
        let test_cases = vec!["us", "US", "Us", "uS"];
        for case in test_cases {
            let req = create_request_with_headers(&[("x-country", case)]);
            assert!(
                is_request_from_country(&req, &app_data, &countries),
                "Should accept US country code in case: '{}'",
                case
            );
        }
    }

    #[actix_web::test]
    async fn test_is_request_from_country_with_whitespace() {
        use hakanai_lib::models::CountryCode;

        let countries = vec![CountryCode::from_str("US").unwrap()];
        let app_data = AppData::default().with_country_header(Some("x-country".to_string()));

        let req = create_request_with_headers(&[("x-country", "  US  ")]);
        assert!(
            is_request_from_country(&req, &app_data, &countries),
            "Should accept US country code with whitespace after trimming"
        );

        // Test invalid country code (after trimming)
        let req = create_request_with_headers(&[("x-country", "  DE  ")]);
        assert!(
            !is_request_from_country(&req, &app_data, &countries),
            "Should reject non-allowed country DE even with whitespace"
        );
    }
}
