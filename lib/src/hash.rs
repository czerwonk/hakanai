use sha2::{Digest, Sha256};

/// Hashes a given string using SHA-256 and returns the hexadecimal representation.
pub fn hash_string(input: &str) -> String {
    return hash_bytes(input.as_bytes());
}

/// Hashes given bytes using SHA-256 and returns the hexadecimal representation.
pub fn hash_bytes(input: &[u8]) -> String {
    let token_hash = Sha256::digest(input);
    format!("{token_hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string_basic() {
        let result = hash_string("hello");
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hash_string_empty() {
        let result = hash_string("");
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_string_consistency() {
        let input = "test_token_123";
        let hash1 = hash_string(input);
        let hash2 = hash_string(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_different_inputs() {
        let hash1 = hash_string("token1");
        let hash2 = hash_string("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_unicode() {
        let result = hash_string("hello 世界 🌍");
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_string_special_characters() {
        let result = hash_string("!@#$%^&*()_+-=[]{}|;':\",./<>?");
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_string_whitespace() {
        let hash1 = hash_string("token");
        let hash2 = hash_string(" token");
        let hash3 = hash_string("token ");
        let hash4 = hash_string(" token ");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
        assert_ne!(hash2, hash4);
        assert_ne!(hash3, hash4);
    }

    #[test]
    fn test_hash_string_long_input() {
        let long_string = "a".repeat(1000);
        let result = hash_string(&long_string);
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_string_known_values() {
        assert_eq!(
            hash_string("password"),
            "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
        );

        assert_eq!(
            hash_string("The quick brown fox jumps over the lazy dog"),
            "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        );
    }

    #[test]
    fn test_hash_string_similar_inputs() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("Test");
        let hash3 = hash_string("tEst");
        let hash4 = hash_string("TEST");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
        assert_ne!(hash2, hash4);
        assert_ne!(hash3, hash4);
    }
}
