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
        Self {
            data: data,
            filename,
        }
    }

    /// Decodes the base64 data and returns it as bytes.
    pub fn decode_bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::prelude::BASE64_STANDARD.decode(&self.data)
    }
}

impl Zeroize for Payload {
    fn zeroize(&mut self) {
        self.data.zeroize();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_from_bytes() {
        let bytes = b"Hello, world!";
        let payload = Payload::from_bytes(bytes, Some("greeting.txt".to_string()));
        assert_eq!(payload.filename, Some("greeting.txt".to_string()));
        assert_eq!(
            payload.data.to_string(),
            base64::prelude::BASE64_STANDARD.encode(bytes)
        );
    }

    #[test]
    fn test_payload_decode_bytes() {
        let bytes = b"Hello, world!";
        let payload = Payload::from_bytes(bytes, None);
        let decoded = payload.decode_bytes().unwrap();
        assert_eq!(decoded, bytes);
    }
}
