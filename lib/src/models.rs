// SPDX-License-Identifier: Apache-2.0

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

/// Represents access restrictions for a secret.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SecretRestrictions {
    /// IP addresses/ranges allowed to access the secret
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_ip_nets")]
    pub allowed_ips: Option<Vec<ipnet::IpNet>>,
}

fn deserialize_ip_nets<'de, D>(deserializer: D) -> Result<Option<Vec<ipnet::IpNet>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    // Handle both Vec<String> and null/missing cases
    let strings_opt = Option::<Vec<String>>::deserialize(deserializer)?;

    match strings_opt {
        Some(strings) => {
            let mut ip_nets = Vec::new();
            for s in strings {
                let ip_net =
                    crate::utils::ip_parser::parse_ipnet(&s).map_err(|e| Error::custom(e))?;
                ip_nets.push(ip_net);
            }
            Ok(Some(ip_nets))
        }
        None => Ok(None),
    }
}

impl SecretRestrictions {
    /// Creates a new SecretRestrictions with IP restrictions
    pub fn with_allowed_ips(allowed_ips: Vec<ipnet::IpNet>) -> Self {
        Self {
            allowed_ips: Some(allowed_ips),
        }
    }

    /// Checks if any restrictions are set
    pub fn is_empty(&self) -> bool {
        self.allowed_ips.is_none()
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

    /// Access restrictions for the secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<SecretRestrictions>,
}

impl PostSecretRequest {
    /// Creates a new `PostSecretRequest`.
    ///
    /// # Arguments
    ///
    /// * `data` - The secret data.
    /// * `expires_in` - The duration until expiration.
    pub fn new(data: String, expires_in: Duration) -> Self {
        Self {
            data,
            expires_in,
            restrictions: None,
        }
    }

    /// Sets access restrictions for the secret
    pub fn with_restrictions(mut self, restrictions: SecretRestrictions) -> Self {
        self.restrictions = Some(restrictions);
        self
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

    #[test]
    fn test_secret_restrictions_deserialization() {
        // Test with valid IP addresses and CIDR ranges
        let json = r#"{"allowed_ips": ["127.0.0.1", "192.168.1.0/24", "::1", "2001:db8::/32"]}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();

        let ips = restrictions.allowed_ips.unwrap();
        assert_eq!(ips.len(), 4);
        assert_eq!(ips[0].to_string(), "127.0.0.1/32");
        assert_eq!(ips[1].to_string(), "192.168.1.0/24");
        assert_eq!(ips[2].to_string(), "::1/128");
        assert_eq!(ips[3].to_string(), "2001:db8::/32");
    }

    #[test]
    fn test_secret_restrictions_deserialization_empty() {
        // Test with null allowed_ips
        let json = r#"{"allowed_ips": null}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());

        // Test with empty object
        let json = r#"{}"#;
        let restrictions: SecretRestrictions = serde_json::from_str(json).unwrap();
        assert!(restrictions.allowed_ips.is_none());
    }

    #[test]
    fn test_secret_restrictions_deserialization_invalid_ip() {
        let json = r#"{"allowed_ips": ["invalid-ip"]}"#;
        let result: std::result::Result<SecretRestrictions, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid IP address or CIDR notation")
        );
    }
}
