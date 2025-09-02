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
        assert_eq!(
            parse("1024"),
            Ok(1024),
            "Standard byte value should parse correctly"
        );
        assert_eq!(parse("0"), Ok(0), "Zero bytes should parse correctly");
        assert_eq!(
            parse("999"),
            Ok(999),
            "Arbitrary byte value should parse correctly"
        );
        assert_eq!(parse("  1024  "), Ok(1024), "Whitespace should be trimmed");
    }

    #[test]
    fn test_parse_kilobytes() {
        assert_eq!(
            parse("1k"),
            Ok(1024),
            "Lowercase k should parse as kilobytes"
        );
        assert_eq!(
            parse("1K"),
            Ok(1024),
            "Uppercase K should parse as kilobytes"
        );
        assert_eq!(
            parse("0.5k"),
            Ok(512),
            "Decimal kilobytes should parse correctly"
        );
        assert_eq!(
            parse("2k"),
            Ok(2048),
            "Multiple kilobytes should parse correctly"
        );
        assert_eq!(
            parse("  1k  "),
            Ok(1024),
            "Whitespace should be trimmed for kilobytes"
        );
    }

    #[test]
    fn test_parse_megabytes() {
        assert_eq!(
            parse("1m"),
            Ok(1048576),
            "Lowercase m should parse as megabytes"
        );
        assert_eq!(
            parse("1M"),
            Ok(1048576),
            "Uppercase M should parse as megabytes"
        );
        assert_eq!(
            parse("0.5m"),
            Ok(524288),
            "Decimal megabytes should parse correctly"
        );
        assert_eq!(
            parse("2m"),
            Ok(2097152),
            "Multiple megabytes should parse correctly"
        );
        assert_eq!(
            parse("  1m  "),
            Ok(1048576),
            "Whitespace should be trimmed for megabytes"
        );
    }

    #[test]
    fn test_parse_decimal_values() {
        assert_eq!(
            parse("1.5k"),
            Ok(1536),
            "Decimal kilobytes should calculate correctly"
        );
        assert_eq!(
            parse("2.25m"),
            Ok(2359296),
            "Decimal megabytes should calculate correctly"
        );
        assert_eq!(
            parse("0.75k"),
            Ok(768),
            "Fractional kilobytes should calculate correctly"
        );
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = parse("invalid");
        assert!(
            result.is_err(),
            "Word 'invalid' should be rejected, got: {:?}",
            result
        );

        let result = parse("1g");
        assert!(
            result.is_err(),
            "Unsupported 'g' suffix should be rejected, got: {:?}",
            result
        );

        let result = parse("1kb");
        assert!(
            result.is_err(),
            "Multi-char 'kb' suffix should be rejected, got: {:?}",
            result
        );

        let result = parse("k");
        assert!(
            result.is_err(),
            "Suffix without number should be rejected, got: {:?}",
            result
        );

        let result = parse("m");
        assert!(
            result.is_err(),
            "Suffix without number should be rejected, got: {:?}",
            result
        );

        let result = parse("");
        assert!(
            result.is_err(),
            "Empty string should be rejected, got: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_invalid_numbers() {
        let result = parse("abc");
        assert!(
            result.is_err(),
            "Non-numeric text should be rejected, got: {:?}",
            result
        );

        let result = parse("1.2.3k");
        assert!(
            result.is_err(),
            "Multiple decimal points should be rejected, got: {:?}",
            result
        );

        let result = parse("--1k");
        assert!(
            result.is_err(),
            "Double negative should be rejected, got: {:?}",
            result
        );

        let result = parse("1..5m");
        assert!(
            result.is_err(),
            "Double decimal points should be rejected, got: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_negative_values() {
        assert_eq!(parse("-1"), Ok(-1), "Negative bytes should parse correctly");
        assert_eq!(
            parse("-1k"),
            Ok(-1024),
            "Negative kilobytes should parse correctly"
        );
        assert_eq!(
            parse("-0.5m"),
            Ok(-524288),
            "Negative decimal megabytes should parse correctly"
        );
    }

    #[test]
    fn test_parse_large_values() {
        assert_eq!(
            parse("1000m"),
            Ok(1048576000),
            "Large megabyte values should parse correctly"
        );
        assert_eq!(
            parse("9999k"),
            Ok(10238976),
            "Large kilobyte values should parse correctly"
        );
    }
}
