use std::net::IpAddr;
use std::str::FromStr;

use actix_web::HttpRequest;

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

/// Extract a trusted header value from the request
fn extract_trusted_header(req: &HttpRequest, header_name: &str) -> Option<String> {
    req.headers()
        .get(header_name)
        .and_then(|h| h.to_str().ok().map(|s| s.trim().to_string()))
}

/// Extract client IP from request headers or connection info
fn extract_client_ip(req: &HttpRequest, trusted_header: &str) -> Option<IpAddr> {
    // First check the configured trusted header (e.g., x-forwarded-for)
    if let Some(header_value) = extract_trusted_header(req, trusted_header) {
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

/// Check if an IP address is in any of the provided ranges
fn is_ip_in_ranges(ip: &IpAddr, ranges: &[ipnet::IpNet]) -> bool {
    ranges.iter().any(|range| range.contains(ip))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_data::{AnonymousOptions, AppData};
    use crate::observer::ObserverManager;
    use crate::test_utils::{MockDataStore, MockTokenManager};
    use actix_web::{HttpRequest, test};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::time::Duration;

    fn create_test_app_data(trusted_ranges: Option<Vec<ipnet::IpNet>>, header: &str) -> AppData {
        AppData {
            data_store: Box::new(MockDataStore::new()),
            token_validator: Box::new(MockTokenManager::new()),
            token_creator: Box::new(MockTokenManager::new()),
            max_ttl: Duration::from_secs(7200),
            anonymous_usage: AnonymousOptions {
                allowed: true,
                upload_size_limit: 32 * 1024,
            },
            impressum_html: None,
            privacy_html: None,
            observer_manager: ObserverManager::new(),
            show_token_input: false,
            trusted_ip_ranges: trusted_ranges,
            trusted_ip_header: header.to_string(),
        }
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
    async fn test_extract_trusted_header() {
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.1")]);
        let header = extract_trusted_header(&req, "x-forwarded-for");
        assert_eq!(header, Some("192.168.1.1".to_string()));

        let header = extract_trusted_header(&req, "nonexistent-header");
        assert_eq!(header, None);
    }

    #[actix_web::test]
    async fn test_extract_trusted_header_with_whitespace() {
        let req = create_request_with_headers(&[("x-forwarded-for", "  192.168.1.1  ")]);
        let header = extract_trusted_header(&req, "x-forwarded-for");
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
    async fn test_is_request_from_whitelisted_ip_false() {
        let ranges = vec!["10.0.0.0/8".parse::<ipnet::IpNet>().unwrap()];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.1")]);

        assert!(!is_request_from_whitelisted_ip(&req, &app_data));
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_no_ranges() {
        let app_data = create_test_app_data(None, "x-forwarded-for");
        let req = create_request_with_headers(&[("x-forwarded-for", "10.0.0.1")]);

        assert!(!is_request_from_whitelisted_ip(&req, &app_data));
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_no_client_ip() {
        let ranges = vec!["10.0.0.0/8".parse::<ipnet::IpNet>().unwrap()];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");
        let req = create_request_with_headers(&[]);

        assert!(!is_request_from_whitelisted_ip(&req, &app_data));
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_different_headers() {
        let ranges = vec!["192.168.1.0/24".parse::<ipnet::IpNet>().unwrap()];

        // Test with x-forwarded-for header
        let app_data = create_test_app_data(Some(ranges.clone()), "x-forwarded-for");
        let req = create_request_with_headers(&[
            ("x-forwarded-for", "192.168.1.100"),
            ("cf-connecting-ip", "10.0.0.1"),
        ]);
        assert!(is_request_from_whitelisted_ip(&req, &app_data));

        // Test with cf-connecting-ip header
        let app_data = create_test_app_data(Some(ranges), "cf-connecting-ip");
        let req = create_request_with_headers(&[
            ("x-forwarded-for", "192.168.1.100"),
            ("cf-connecting-ip", "10.0.0.1"),
        ]);
        assert!(!is_request_from_whitelisted_ip(&req, &app_data));
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_localhost() {
        let ranges = vec![
            "::1/128".parse::<ipnet::IpNet>().unwrap(),
            "127.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
        ];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");

        // Test IPv4 localhost
        let req = create_request_with_headers(&[("x-forwarded-for", "127.0.0.1")]);
        assert!(is_request_from_whitelisted_ip(&req, &app_data));

        // Test IPv6 localhost
        let req = create_request_with_headers(&[("x-forwarded-for", "::1")]);
        assert!(is_request_from_whitelisted_ip(&req, &app_data));
    }

    #[actix_web::test]
    async fn test_is_request_from_whitelisted_ip_edge_cases() {
        let ranges = vec!["192.168.1.0/24".parse::<ipnet::IpNet>().unwrap()];
        let app_data = create_test_app_data(Some(ranges), "x-forwarded-for");

        // Test network address (first IP in range)
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.0")]);
        assert!(is_request_from_whitelisted_ip(&req, &app_data));

        // Test broadcast address (last IP in range)
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.1.255")]);
        assert!(is_request_from_whitelisted_ip(&req, &app_data));

        // Test just outside range
        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.2.0")]);
        assert!(!is_request_from_whitelisted_ip(&req, &app_data));

        let req = create_request_with_headers(&[("x-forwarded-for", "192.168.0.255")]);
        assert!(!is_request_from_whitelisted_ip(&req, &app_data));
    }
}
