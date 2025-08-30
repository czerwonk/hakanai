// SPDX-License-Identifier: Apache-2.0

use base64::Engine;
use sha2::{Digest, Sha256};

/// Hashes a given string using SHA-256 and returns the hexadecimal representation.
pub fn sha256_hex_from_string(input: &str) -> String {
    sha256_hex_from_bytes(input.as_bytes())
}

/// Hashes given bytes using SHA-256 and returns the hexadecimal representation.
pub fn sha256_hex_from_bytes(input: &[u8]) -> String {
    let hash = Sha256::digest(input);
    format!("{hash:x}")
}

/// Hashes given bytes using SHA-256, truncates the result to the first 16 bytes. Result is then encoded to a URL-safe base64 string without padding.
pub fn sha256_truncated_base64_from_bytes(input: &[u8]) -> String {
    let hash = Sha256::digest(input);
    base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&hash[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex_from_string_basic() {
        let result = sha256_hex_from_string("hello");
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha256_hex_from_string_empty() {
        let result = sha256_hex_from_string("");
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hex_from_string_consistency() {
        let input = "test_token_123";
        let hash1 = sha256_hex_from_string(input);
        let hash2 = sha256_hex_from_string(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hex_from_string_different_inputs() {
        let hash1 = sha256_hex_from_string("token1");
        let hash2 = sha256_hex_from_string("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hex_from_string_unicode() {
        let result = sha256_hex_from_string("hello 世界 🌍");
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex_from_string_special_characters() {
        let result = sha256_hex_from_string("!@#$%^&*()_+-=[]{}|;':\",./<>?");
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex_from_string_whitespace() {
        let hash1 = sha256_hex_from_string("token");
        let hash2 = sha256_hex_from_string(" token");
        let hash3 = sha256_hex_from_string("token ");
        let hash4 = sha256_hex_from_string(" token ");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
        assert_ne!(hash2, hash4);
        assert_ne!(hash3, hash4);
    }

    #[test]
    fn test_sha256_hex_from_string_long_input() {
        let long_string = "a".repeat(1000);
        let result = sha256_hex_from_string(&long_string);
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex_from_string_known_values() {
        assert_eq!(
            sha256_hex_from_string("password"),
            "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
        );

        assert_eq!(
            sha256_hex_from_string("The quick brown fox jumps over the lazy dog"),
            "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        );
    }

    #[test]
    fn test_sha256_hex_from_string_similar_inputs() {
        let hash1 = sha256_hex_from_string("test");
        let hash2 = sha256_hex_from_string("Test");
        let hash3 = sha256_hex_from_string("tEst");
        let hash4 = sha256_hex_from_string("TEST");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
        assert_ne!(hash2, hash4);
        assert_ne!(hash3, hash4);
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_basic() {
        let result = sha256_truncated_base64_from_bytes(b"hello");
        // Should be 22 characters (16 bytes * 4/3, rounded up, no padding)
        assert_eq!(result.len(), 22);
        // Should only contain URL-safe base64 characters
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        // Should not contain padding
        assert!(!result.contains('='));
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_empty() {
        let result = sha256_truncated_base64_from_bytes(b"");
        assert_eq!(result.len(), 22);
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!result.contains('='));
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_consistency() {
        let input = b"test_token_123";
        let hash1 = sha256_truncated_base64_from_bytes(input);
        let hash2 = sha256_truncated_base64_from_bytes(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_different_inputs() {
        let hash1 = sha256_truncated_base64_from_bytes(b"token1");
        let hash2 = sha256_truncated_base64_from_bytes(b"token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_unicode() {
        let input = "hello 世界 🌍".as_bytes();
        let result = sha256_truncated_base64_from_bytes(input);
        assert_eq!(result.len(), 22);
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!result.contains('='));
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_special_characters() {
        let input = b"!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let result = sha256_truncated_base64_from_bytes(input);
        assert_eq!(result.len(), 22);
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!result.contains('='));
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_whitespace() {
        let hash1 = sha256_truncated_base64_from_bytes(b"token");
        let hash2 = sha256_truncated_base64_from_bytes(b" token");
        let hash3 = sha256_truncated_base64_from_bytes(b"token ");
        let hash4 = sha256_truncated_base64_from_bytes(b" token ");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
        assert_ne!(hash2, hash4);
        assert_ne!(hash3, hash4);
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_long_input() {
        let long_bytes = vec![b'a'; 1000];
        let result = sha256_truncated_base64_from_bytes(&long_bytes);
        assert_eq!(result.len(), 22);
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!result.contains('='));
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_known_values() {
        // Test with known SHA-256 values to verify correct truncation and encoding

        // "hello" SHA-256: 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        // First 16 bytes: 2cf24dba5fb0a30e26e83b2ac5b9e29e
        // Expected base64url: LPJNul-wow4m6Dsqxbning
        let result = sha256_truncated_base64_from_bytes(b"hello");
        assert_eq!(result, "LPJNul-wow4m6Dsqxbning");

        // "password" SHA-256: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
        // First 16 bytes: 5e884898da28047151d0e56f8dc62927
        // Expected base64url: XohImNooBHFR0OVvjcYpJw
        let result2 = sha256_truncated_base64_from_bytes(b"password");
        assert_eq!(result2, "XohImNooBHFR0OVvjcYpJw");

        // Empty string SHA-256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        // First 16 bytes: e3b0c44298fc1c149afbf4c8996fb924
        // Expected base64url: 47DEQpj8HBSa-_TImW-5JA
        let result3 = sha256_truncated_base64_from_bytes(b"");
        assert_eq!(result3, "47DEQpj8HBSa-_TImW-5JA");

        // Verify all results have correct format
        for result in [&result, &result2, &result3] {
            assert_eq!(result.len(), 22);
            assert!(
                result
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            );
            assert!(!result.contains('='));
            assert!(!result.contains('+'));
            assert!(!result.contains('/'));
        }
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_similar_inputs() {
        let hash1 = sha256_truncated_base64_from_bytes(b"test");
        let hash2 = sha256_truncated_base64_from_bytes(b"Test");
        let hash3 = sha256_truncated_base64_from_bytes(b"tEst");
        let hash4 = sha256_truncated_base64_from_bytes(b"TEST");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
        assert_ne!(hash2, hash4);
        assert_ne!(hash3, hash4);
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_binary_data() {
        let binary_data = vec![0u8, 1, 2, 3, 255, 254, 253, 128, 127];
        let result = sha256_truncated_base64_from_bytes(&binary_data);
        assert_eq!(result.len(), 22);
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!result.contains('='));
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_url_safe_encoding() {
        // Test multiple inputs to ensure we get URL-safe characters
        let inputs = [
            b"hello world".as_slice(),
            b"test123",
            b"!@#$%^&*()",
            b"abcdefghijklmnopqrstuvwxyz",
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            b"0123456789",
        ];

        for input in inputs {
            let result = sha256_truncated_base64_from_bytes(input);
            // Should not contain standard base64 characters that are not URL-safe
            assert!(!result.contains('+'));
            assert!(!result.contains('/'));
            assert!(!result.contains('='));
            // Should only contain URL-safe characters
            assert!(
                result
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            );
        }
    }

    #[test]
    fn test_sha256_truncated_base64_from_bytes_truncation() {
        // Verify that the function actually truncates (different from full hash)
        let input = b"test";
        let truncated = sha256_truncated_base64_from_bytes(input);
        let full_hash = sha256_hex_from_bytes(input);

        // Truncated should be much shorter (22 chars vs 64 chars)
        assert_eq!(truncated.len(), 22);
        assert_eq!(full_hash.len(), 64);

        // They should be different representations
        assert_ne!(truncated, full_hash);
    }
}
