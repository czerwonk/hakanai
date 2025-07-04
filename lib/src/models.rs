use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Represents the data payload of a secret, which can be either a text message
/// or a file with optional metadata.
#[derive(Deserialize, Serialize)]
pub struct Payload {
    /// The base64-encoded data of the secret.
    pub data: String,

    /// The filename of the file, if not set data is assumed to be a text message.
    pub filename: Option<String>,

    size_of: usize,
}

impl Payload {
    /// Creates a new `Payload` instance from raw text data.
    ///
    /// # Arguments
    ///
    /// * `text` - The raw text data of the secret.
    pub fn from_text(text: &str) -> Self {
        use base64::Engine;
        let data = base64::prelude::BASE64_STANDARD.encode(text.as_bytes());
        Self {
            data,
            filename: None,
            size_of: text.len(),
        }
    }

    /// Creates a new `Payload` instance from raw binary data.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw binary data of the secret.
    /// * `filename` - An optional filename for the file.
    pub fn from_bytes(bytes: &[u8], filename: Option<String>) -> Self {
        use base64::Engine;
        let data = base64::prelude::BASE64_STANDARD.encode(bytes);
        Self {
            data,
            filename,
            size_of: bytes.len(),
        }
    }

    /// Decodes the base64 data and returns it as bytes.
    pub fn decode_bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::Engine;
        base64::prelude::BASE64_STANDARD.decode(&self.data)
    }

    /// Decodes the base64 data and returns it as a UTF-8 string.
    pub fn decode_text(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.decode_bytes()?;
        Ok(String::from_utf8(bytes)?)
    }

    /// Checks if the payload is empty.
    pub fn is_empty(&self) -> bool {
        self.size_of == 0
    }
}

/// Represents the request to create a new secret.
#[serde_as]
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
