// SPDX-License-Identifier: Apache-2.0

use base64::Engine;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Represents the data payload of a secret, which can be either a text message
/// or a file with optional metadata.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Payload {
    /// The base64-encoded data of the secret.
    pub data: String,

    /// The filename of the file, if not set data is assumed to be a text message.
    pub filename: Option<String>,
}

impl Payload {
    /// Creates a new `Payload` instance from raw binary data.
    ///
    /// This method automatically base64-encodes the input bytes, making it suitable
    /// for transmitting binary files or data that may contain non-UTF8 sequences.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw binary data of the secret.
    /// * `filename` - An optional filename for the file.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let data = base64::prelude::BASE64_STANDARD.encode(bytes);
        Self {
            data,
            filename: None,
        }
    }

    /// Sets the filename for the payload, indicating that it represents a file.
    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
        self
    }

    /// Decodes the base64 data and returns it as bytes.
    pub fn decode_bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::prelude::BASE64_STANDARD.decode(&self.data)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl Zeroize for Payload {
    fn zeroize(&mut self) {
        self.data.zeroize();
        if let Some(ref mut filename) = self.filename {
            filename.zeroize();
        }
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[test]
    fn test_payload_from_bytes() -> Result<()> {
        let bytes = b"Hello, world!";
        let payload = Payload::from_bytes(bytes).with_filename("greeting.txt");
        assert_eq!(payload.filename, Some("greeting.txt".to_string()));
        assert_eq!(
            payload.data.to_string(),
            base64::prelude::BASE64_STANDARD.encode(bytes)
        );
        Ok(())
    }

    #[test]
    fn test_payload_decode_bytes() -> Result<()> {
        let bytes = b"Hello, world!";
        let payload = Payload::from_bytes(bytes);
        let decoded = payload.decode_bytes()?;
        assert_eq!(decoded, bytes);
        Ok(())
    }

    #[test]
    fn test_payload_serialization_roundtrip() -> Result<()> {
        let payload = Payload::from_bytes(b"test data").with_filename("test.txt");

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, payload.data);
        assert_eq!(deserialized.filename, payload.filename);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_text_no_filename() -> Result<()> {
        let bytes = base64::prelude::BASE64_STANDARD
            .encode(b"Hello, world!")
            .as_bytes()
            .to_vec();
        let payload = Payload::from_bytes(&bytes);

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, payload.data);
        assert_eq!(deserialized.filename, None);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_empty() -> Result<()> {
        let payload = Payload::from_bytes(b"");

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, "");
        assert_eq!(deserialized.filename, None);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_special_filename_characters() -> Result<()> {
        let special_filenames = vec![
            "file with spaces.txt",
            "file-with-dashes.pdf",
            "file_with_underscores.doc",
            "file.with.multiple.dots.tar.gz",
            "ファイル.txt", // Japanese characters
            "файл.txt",     // Cyrillic characters
            "🎉emoji.txt",  // Emoji
            "file@symbol.txt",
            "file#hash.txt",
            "file$dollar.txt",
            "file%percent.txt",
            "file&ampersand.txt",
            "file(parentheses).txt",
            "file[brackets].txt",
            "file{braces}.txt",
            "file+plus.txt",
            "file=equals.txt",
            "file'quote.txt",
            "file\"doublequote.txt",
        ];

        for filename in special_filenames {
            let bytes = base64::prelude::BASE64_STANDARD
                .encode(b"test content")
                .as_bytes()
                .to_vec();
            let payload = Payload::from_bytes(&bytes).with_filename(filename);

            let serialized = payload.serialize()?;
            let deserialized = Payload::deserialize(&serialized)?;

            assert_eq!(deserialized.filename, Some(filename.to_string()));
        }
        Ok(())
    }

    #[test]
    fn test_payload_serialize_large_binary() -> Result<()> {
        // Create a large binary payload (1MB)
        let large_data = vec![0xDEu8; 1024 * 1024];
        let payload = Payload::from_bytes(&large_data).with_filename("large_file.bin");

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.filename, Some("large_file.bin".to_string()));
        let decoded = deserialized.decode_bytes()?;
        assert_eq!(decoded.len(), large_data.len());
        assert_eq!(decoded, large_data);
        Ok(())
    }

    #[test]
    fn test_payload_deserialize_invalid_json() {
        let invalid_json = b"{ invalid json }";
        let result = Payload::deserialize(invalid_json);
        assert!(
            result.is_err(),
            "Expected error for invalid JSON, got: {:?}",
            result
        );
    }

    #[test]
    fn test_payload_deserialize_missing_fields() {
        // JSON with missing required field
        let incomplete_json = br#"{"filename": "test.txt"}"#;
        let result = Payload::deserialize(incomplete_json);
        assert!(
            result.is_err(),
            "Expected error for missing fields, got: {:?}",
            result
        );
    }

    #[test]
    fn test_payload_deserialize_wrong_type() {
        // JSON with wrong type for data field
        let wrong_type_json = br#"{"data": 123, "filename": "test.txt"}"#;
        let result = Payload::deserialize(wrong_type_json);
        assert!(
            result.is_err(),
            "Expected error for wrong type, got: {:?}",
            result
        );
    }

    #[test]
    fn test_payload_from_bytes_without_filename() -> Result<()> {
        let bytes = b"Secret message";
        let payload = Payload::from_bytes(bytes);

        assert!(payload.filename.is_none());
        assert_eq!(payload.decode_bytes()?, bytes);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_preserves_base64() -> Result<()> {
        // Test that serialization preserves the exact base64 encoding
        let original_bytes = b"Binary\x00\x01\x02\x03data";
        let payload = Payload::from_bytes(original_bytes).with_filename("binary.dat");

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(payload.data, deserialized.data);
        assert_eq!(deserialized.decode_bytes()?, original_bytes);
        Ok(())
    }

    #[test]
    fn test_payload_zeroize() {
        let mut payload = Payload::from_bytes(b"sensitive data").with_filename("secret.txt");

        payload.zeroize();

        assert_eq!(payload.data, "");
        assert_eq!(payload.filename, Some("".to_string()));
    }
}
