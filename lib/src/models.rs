// SPDX-License-Identifier: MIT

use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
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
    ///
    /// # Examples
    ///
    /// ```
    /// use hakanai_lib::models::Payload;
    ///
    /// // Binary file
    /// let pdf_bytes = vec![0x25, 0x50, 0x44, 0x46]; // PDF magic bytes
    /// let payload = Payload::from_bytes(&pdf_bytes, Some("document.pdf".to_string()));
    /// assert_eq!(payload.filename, Some("document.pdf".to_string()));
    ///
    /// // Text without filename
    /// let text_payload = Payload::from_bytes(b"Secret message", None);
    /// assert!(text_payload.filename.is_none());
    /// ```
    pub fn from_bytes(bytes: &[u8], filename: Option<String>) -> Self {
        let data = base64::prelude::BASE64_STANDARD.encode(bytes);
        Self { data, filename }
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

/// Represents the request to create a new secret.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostSecretRequest {
    /// The secret data to be stored.
    pub data: String,

    /// The duration until the secret expires.
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub expires_in: Duration,
}

impl PostSecretRequest {
    /// Creates a new `PostSecretRequest`.
    ///
    /// # Arguments
    ///
    /// * `data` - The secret data.
    /// * `expires_in` - The duration until expiration.
    pub fn new(data: String, expires_in: Duration) -> Self {
        Self { data, expires_in }
    }
}

/// Represents the response after creating a new secret.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostSecretResponse {
    /// The unique identifier of the created secret.
    pub id: uuid::Uuid,
}

impl PostSecretResponse {
    /// Creates a new `PostSecretResponse`.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the secret.
    pub fn new(id: uuid::Uuid) -> Self {
        Self { id }
    }
}

/// Request model for creating user tokens via admin API
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenRequest {
    /// Optional upload size limit in bytes
    pub upload_size_limit: Option<i64>,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

/// Response model for creating user tokens via admin API
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenResponse {
    /// The generated token
    pub token: String,
}

impl Zeroize for CreateTokenResponse {
    fn zeroize(&mut self) {
        self.token.zeroize();
    }
}

impl Drop for CreateTokenResponse {
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
        let payload = Payload::from_bytes(bytes, Some("greeting.txt".to_string()));
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
        let payload = Payload::from_bytes(bytes, None);
        let decoded = payload.decode_bytes()?;
        assert_eq!(decoded, bytes);
        Ok(())
    }

    #[test]
    fn test_payload_serialization_roundtrip() -> Result<()> {
        let payload = Payload {
            data: "test data".to_string(),
            filename: Some("test.txt".to_string()),
        };

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, payload.data);
        assert_eq!(deserialized.filename, payload.filename);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_text_no_filename() -> Result<()> {
        let payload = Payload {
            data: base64::prelude::BASE64_STANDARD.encode(b"Hello, world!"),
            filename: None,
        };

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, payload.data);
        assert_eq!(deserialized.filename, None);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_empty() -> Result<()> {
        let payload = Payload {
            data: String::new(),
            filename: None,
        };

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
            let payload = Payload {
                data: base64::prelude::BASE64_STANDARD.encode(b"test content"),
                filename: Some(filename.to_string()),
            };

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
        let payload = Payload::from_bytes(&large_data, Some("large_file.bin".to_string()));

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
        assert!(result.is_err());
    }

    #[test]
    fn test_payload_deserialize_missing_fields() {
        // JSON with missing required field
        let incomplete_json = br#"{"filename": "test.txt"}"#;
        let result = Payload::deserialize(incomplete_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_payload_deserialize_wrong_type() {
        // JSON with wrong type for data field
        let wrong_type_json = br#"{"data": 123, "filename": "test.txt"}"#;
        let result = Payload::deserialize(wrong_type_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_payload_from_bytes_without_filename() -> Result<()> {
        let bytes = b"Secret message";
        let payload = Payload::from_bytes(bytes, None);

        assert!(payload.filename.is_none());
        assert_eq!(payload.decode_bytes()?, bytes);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_preserves_base64() -> Result<()> {
        // Test that serialization preserves the exact base64 encoding
        let original_bytes = b"Binary\x00\x01\x02\x03data";
        let payload = Payload::from_bytes(original_bytes, Some("binary.dat".to_string()));

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(payload.data, deserialized.data);
        assert_eq!(deserialized.decode_bytes()?, original_bytes);
        Ok(())
    }

    #[test]
    fn test_payload_zeroize() {
        let mut payload = Payload {
            data: "sensitive data".to_string(),
            filename: Some("secret.txt".to_string()),
        };

        payload.zeroize();

        assert_eq!(payload.data, "");
        assert_eq!(payload.filename, Some("".to_string()));
    }
}
