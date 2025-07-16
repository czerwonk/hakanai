//! Content analysis utilities for detecting binary vs text data.
//!
//! This module provides functions to analyze byte content and determine
//! whether it represents binary or text data. This is useful for:
//! - Deciding how to display content in user interfaces
//! - Determining appropriate encoding/decoding strategies
//! - Validating input data types
//!
//! # Example
//!
//! ```
//! use hakanai_lib::utils::content_analysis::is_binary;
//!
//! let text_data = b"Hello, world!";
//! let binary_data = b"\x00\x01\x02\xFF";
//!
//! assert!(!is_binary(text_data));
//! assert!(is_binary(binary_data));
//! ```

/// Checks if the given content is binary data.
///
/// This function uses a simple but effective heuristic: the presence of null bytes.
/// Most text encodings (UTF-8, ASCII, etc.) don't contain null bytes, while binary
/// formats (executables, images, compressed files) commonly do.
///
/// # Arguments
///
/// * `content` - A byte slice to analyze
///
/// # Returns
///
/// * `true` if the content appears to be binary data
/// * `false` if the content appears to be text
///
/// # Limitations
///
/// This is a heuristic approach and may have edge cases:
/// - Some text files with special encodings might contain null bytes
/// - Some binary formats might not contain null bytes in their header
///
/// For more robust detection, consider additional checks like UTF-8 validation
/// or magic byte detection for specific file formats.
///
/// # Example
///
/// ```
/// use hakanai_lib::utils::content_analysis::is_binary;
///
/// // Text content
/// assert!(!is_binary(b"Hello, world!"));
/// assert!(!is_binary(b"UTF-8 text: \xE2\x9C\x93")); // ✓
///
/// // Binary content
/// assert!(is_binary(b"\x00\x01\x02"));
/// assert!(is_binary(b"PNG\x00header"));
/// ```
pub fn is_binary(content: &[u8]) -> bool {
    // Check for null bytes, which are common in binary files
    content.contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary_with_text() {
        let text_content = b"Hello, world!";
        assert!(
            !is_binary(text_content),
            "Text content should not be binary"
        );
    }

    #[test]
    fn test_is_binary_with_binary() {
        let binary_content = b"\x00\x01\x02Hello\x00world!";
        assert!(
            is_binary(binary_content),
            "Binary content should be detected as binary"
        );
    }
}
