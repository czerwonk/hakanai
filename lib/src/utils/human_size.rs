// SPDX-License-Identifier: Apache-2.0

//! Size parsing utilities for human-readable input.
//!
//! This module provides functionality to parse size values from human-readable strings,
//! supporting bytes, kilobytes, and megabytes with intuitive suffix notation.

/// Parse a size limit from a human-readable string.
///
/// This function converts human-readable size strings into byte values, supporting
/// multiple formats for user convenience.
///
/// # Supported Formats
///
/// - **Plain numbers**: Interpreted as bytes (e.g., `"1024"` → 1024 bytes)
/// - **Kilobytes**: Numbers followed by 'k' or 'K' (e.g., `"1k"` → 1024 bytes)
/// - **Megabytes**: Numbers followed by 'm' or 'M' (e.g., `"1m"` → 1048576 bytes)
/// - **Decimal values**: Fractional numbers are supported (e.g., `"1.5k"` → 1536 bytes)
/// - **Whitespace**: Leading and trailing whitespace is ignored
///
/// # Arguments
///
/// * `s` - A string slice containing the size specification
///
/// # Returns
///
/// * `Ok(i64)` - The parsed size in bytes
/// * `Err(String)` - An error message describing why parsing failed
///
/// # Examples
///
/// ```
/// use hakanai_lib::utils::human_size::parse;
///
/// // Plain bytes
/// assert_eq!(parse("1024"), Ok(1024));
/// assert_eq!(parse("0"), Ok(0));
///
/// // Kilobytes
/// assert_eq!(parse("1k"), Ok(1024));
/// assert_eq!(parse("2K"), Ok(2048));
/// assert_eq!(parse("1.5k"), Ok(1536));
///
/// // Megabytes
/// assert_eq!(parse("1m"), Ok(1048576));
/// assert_eq!(parse("2M"), Ok(2097152));
/// assert_eq!(parse("0.5m"), Ok(524288));
///
/// // Whitespace handling
/// assert_eq!(parse("  1k  "), Ok(1024));
///
/// // Error cases
/// assert!(parse("invalid").is_err());
/// assert!(parse("1g").is_err());
/// assert!(parse("").is_err());
/// ```
///
/// # Error Messages
///
/// The function returns descriptive error messages for invalid input:
/// - `"Invalid number format"` - When the numeric part cannot be parsed
/// - `"Invalid size format. Use plain bytes, 'k' for KB, or 'm' for MB"` - For unsupported formats
///
/// # Note
///
/// This function uses binary (1024-based) units rather than decimal (1000-based) units,
/// which is common in computing contexts:
/// - 1k = 1024 bytes
/// - 1m = 1024 × 1024 = 1048576 bytes
pub fn parse(s: &str) -> Result<i64, String> {
    let s = s.trim().to_lowercase();

    // Handle plain numbers (assume bytes)
    if let Ok(bytes) = s.parse::<i64>() {
        return Ok(bytes);
    }

    // Handle with suffix
    if s.ends_with('k') {
        let number_str = &s[..s.len() - 1];
        let number: f64 = number_str
            .parse()
            .map_err(|_| "Invalid number format".to_string())?;
        Ok((number * 1024.0) as i64)
    } else if s.ends_with('m') {
        let number_str = &s[..s.len() - 1];
        let number: f64 = number_str
            .parse()
            .map_err(|_| "Invalid number format".to_string())?;
        Ok((number * 1024.0 * 1024.0) as i64)
    } else {
        Err("Invalid size format. Use plain bytes, 'k' for KB, or 'm' for MB".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bytes() {
        assert_eq!(parse("1024"), Ok(1024));
        assert_eq!(parse("0"), Ok(0));
        assert_eq!(parse("999"), Ok(999));
        assert_eq!(parse("  1024  "), Ok(1024));
    }

    #[test]
    fn test_parse_kilobytes() {
        assert_eq!(parse("1k"), Ok(1024));
        assert_eq!(parse("1K"), Ok(1024));
        assert_eq!(parse("0.5k"), Ok(512));
        assert_eq!(parse("2k"), Ok(2048));
        assert_eq!(parse("  1k  "), Ok(1024));
    }

    #[test]
    fn test_parse_megabytes() {
        assert_eq!(parse("1m"), Ok(1048576));
        assert_eq!(parse("1M"), Ok(1048576));
        assert_eq!(parse("0.5m"), Ok(524288));
        assert_eq!(parse("2m"), Ok(2097152));
        assert_eq!(parse("  1m  "), Ok(1048576));
    }

    #[test]
    fn test_parse_decimal_values() {
        assert_eq!(parse("1.5k"), Ok(1536));
        assert_eq!(parse("2.25m"), Ok(2359296));
        assert_eq!(parse("0.75k"), Ok(768));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(
            parse("invalid").is_err(),
            "Expected error for 'invalid', got: {:?}",
            parse("invalid")
        );
        assert!(
            parse("1g").is_err(),
            "Expected error for '1g', got: {:?}",
            parse("1g")
        );
        assert!(
            parse("1kb").is_err(),
            "Expected error for '1kb', got: {:?}",
            parse("1kb")
        );
        assert!(
            parse("k").is_err(),
            "Expected error for 'k', got: {:?}",
            parse("k")
        );
        assert!(
            parse("m").is_err(),
            "Expected error for 'm', got: {:?}",
            parse("m")
        );
        assert!(
            parse("").is_err(),
            "Expected error for empty string, got: {:?}",
            parse("")
        );
    }

    #[test]
    fn test_parse_invalid_numbers() {
        assert!(
            parse("abc").is_err(),
            "Expected error for 'abc', got: {:?}",
            parse("abc")
        );
        assert!(
            parse("1.2.3k").is_err(),
            "Expected error for '1.2.3k', got: {:?}",
            parse("1.2.3k")
        );
        assert!(
            parse("--1k").is_err(),
            "Expected error for '--1k', got: {:?}",
            parse("--1k")
        );
        assert!(
            parse("1..5m").is_err(),
            "Expected error for '1..5m', got: {:?}",
            parse("1..5m")
        );
    }

    #[test]
    fn test_parse_negative_values() {
        assert_eq!(parse("-1"), Ok(-1));
        assert_eq!(parse("-1k"), Ok(-1024));
        assert_eq!(parse("-0.5m"), Ok(-524288));
    }

    #[test]
    fn test_parse_large_values() {
        assert_eq!(parse("1000m"), Ok(1048576000));
        assert_eq!(parse("9999k"), Ok(10238976));
    }
}
