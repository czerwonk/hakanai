// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};

use async_trait::async_trait;
use base64::Engine;
use reqwest::Url;
use zeroize::{Zeroize, Zeroizing};

use crate::client::{Client, ClientError};
use crate::models::Payload;
use crate::options::{SecretReceiveOptions, SecretSendOptions};
use crate::utils::hashing;

const AES_GCM_KEY_SIZE: usize = 32; // AES-256 requires a 32-byte key
const AES_GCM_NONCE_SIZE: usize = 12; // AES-GCM uses a 12-byte nonce

struct CryptoContext {
    key: Vec<u8>,
    nonce: Vec<u8>,
    used: bool,
}

impl CryptoContext {
    /// Creates a new `CryptoContext` with a randomly generated key and nonce.
    fn generate() -> Self {
        let mut key = Zeroizing::new([0u8; AES_GCM_KEY_SIZE]);
        OsRng.fill_bytes(key.as_mut_slice());

        let mut nonce = Zeroizing::new([0u8; AES_GCM_NONCE_SIZE]);
        OsRng.fill_bytes(nonce.as_mut_slice());

        CryptoContext {
            key: key.to_vec(),
            nonce: nonce.to_vec(),
            used: false,
        }
    }

    /// Creates a `CryptoContext` from a base64-encoded fragment.
    fn from_key_base64(fragment: &str) -> Result<Self, ClientError> {
        let key = Zeroizing::new(base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(fragment)?);
        if key.len() != AES_GCM_KEY_SIZE {
            return Err(ClientError::CryptoError("Invalid key length".to_string()));
        }

        Ok(Self {
            key: key.to_vec(),
            nonce: vec![0; AES_GCM_NONCE_SIZE], // nonce will be set later
            used: false,
        })
    }

    fn key_as_base64(&self) -> String {
        base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&self.key)
    }

    fn import_nonce(&mut self, payload: &[u8]) -> Result<(), ClientError> {
        if payload.len() < AES_GCM_NONCE_SIZE {
            return Err(ClientError::CryptoError("Payload too short".to_string()));
        }

        let nonce = &payload[..AES_GCM_NONCE_SIZE];
        self.nonce = nonce.to_vec();
        Ok(())
    }

    fn prepend_nonce_to_ciphertext(&self, ciphertext: &[u8]) -> Vec<u8> {
        let mut result = self.nonce.to_vec();
        result.extend_from_slice(ciphertext);
        result
    }

    fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, ClientError> {
        if self.used {
            return Err(ClientError::CryptoError(
                "CryptoContext has already been used for encryption. Create a new context to prevent nonce reuse."
                    .to_string(),
            ));
        }

        // Mark context as used to prevent nonce reuse
        self.used = true;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.key.as_ref()));
        let nonce = Nonce::from_slice(&self.nonce);
        Ok(cipher.encrypt(nonce, plaintext)?)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ClientError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(self.key.as_ref()));
        let nonce = Nonce::from_slice(&self.nonce);
        Ok(cipher.decrypt(nonce, ciphertext)?)
    }

    #[cfg(test)]
    fn key(&self) -> &[u8] {
        &self.key
    }
}

impl Zeroize for CryptoContext {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.nonce.zeroize();
        self.used = false;
    }
}

impl Drop for CryptoContext {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// A client that wraps another `Client` to provide cryptographic functionalities.
///
/// This struct is responsible for encrypting data before sending and decrypting
/// it upon reception, ensuring that secrets are transmitted securely.
///
/// The `CryptoClient` uses AES-256-GCM for authenticated encryption and embeds
/// the encryption key and content hash in the URL fragment (`#key:hash`) for secure sharing
/// with automatic tamper detection.
///
/// **Note:** This is an internal implementation detail. Users should use `client::new()`
/// which returns a client with encryption already configured.
///
/// # Examples
///
/// ## Basic Usage (Client Library Pattern)
///
/// ```
/// use hakanai_lib::{client, client::Client, models::Payload};
/// use std::time::Duration;
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get a client with encryption built-in
/// let client = client::new();
///
/// // Send a secret (automatically encrypted)
/// let secret_url = client.send_secret(
///     Url::parse("https://api.example.com")?,
///     Payload {
///         data: "My secret message".to_string(),
///         filename: None,
///     },
///     Duration::from_secs(3600),
///     "auth-token".to_string(),
///     None,
/// ).await?;
///
/// // The URL contains the encryption key and hash in the fragment (#key:hash)
/// println!("Share this URL: {}", secret_url);
///
/// // Receive the secret (automatically decrypted)
/// let received = client.receive_secret(secret_url, None).await?;
/// assert_eq!(received.data, "My secret message");
/// # Ok(())
/// # }
/// ```
pub struct CryptoClient {
    inner_client: Box<dyn Client<Vec<u8>>>,
}

impl CryptoClient {
    /// Creates a new instance of `CryptoClient`.
    pub fn new(inner_client: Box<dyn Client<Vec<u8>>>) -> Self {
        CryptoClient { inner_client }
    }
}

#[async_trait]
impl Client<Payload> for CryptoClient {
    async fn send_secret(
        &self,
        base_url: Url,
        payload: Payload,
        ttl: Duration,
        token: String,
        opts: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError> {
        let mut crypto_context = CryptoContext::generate();

        let data = Zeroizing::new(payload.serialize()?);
        let hash = hashing::sha256_truncated_base64_from_bytes(&data);

        let ciphertext = crypto_context.encrypt(&data)?;

        let payload = crypto_context.prepend_nonce_to_ciphertext(&ciphertext);

        let encoded_data = base64::prelude::BASE64_STANDARD
            .encode(&payload)
            .as_bytes()
            .to_vec();

        let res = self
            .inner_client
            .send_secret(base_url, encoded_data, ttl, token, opts)
            .await?;

        let url = append_to_link(res, &crypto_context, &hash);

        Ok(url)
    }

    async fn receive_secret(
        &self,
        url: Url,
        opts: Option<SecretReceiveOptions>,
    ) -> Result<Payload, ClientError> {
        let parts = url
            .fragment()
            .ok_or(ClientError::Custom("No key in URL".to_string()))?
            .split(':')
            .collect::<Vec<&str>>();

        let crypto_context = CryptoContext::from_key_base64(parts[0])?;
        let hash = parts
            .get(1)
            .ok_or(ClientError::Custom(
                "Missing hash in URL fragment".to_string(),
            ))?
            .to_string();

        let encoded_data = self.inner_client.receive_secret(url, opts).await?;
        decrypt(encoded_data, crypto_context, hash)
    }
}

fn append_to_link(url: Url, crypto_context: &CryptoContext, hash: &str) -> Url {
    let mut link = url.clone();

    let mut fragment = crypto_context.key_as_base64();
    fragment.push_str(&format!(":{hash}"));

    link.set_fragment(Some(&fragment));
    fragment.zeroize();

    link
}

fn decrypt(
    encoded_data: Vec<u8>,
    mut crypto_context: CryptoContext,
    hash: String,
) -> Result<Payload, ClientError> {
    let payload = Zeroizing::new(base64::prelude::BASE64_STANDARD.decode(encoded_data)?);

    crypto_context.import_nonce(&payload)?;
    let ciphertext = &payload[AES_GCM_NONCE_SIZE..];
    let plaintext = Zeroizing::new(crypto_context.decrypt(ciphertext)?);

    verify_hash(&plaintext, &hash)?;

    let payload = Payload::deserialize(&plaintext)?;
    Ok(payload)
}

fn verify_hash(plaintext: &[u8], expected_hash: &str) -> Result<(), ClientError> {
    let actual_hash = hashing::sha256_truncated_base64_from_bytes(plaintext);
    if actual_hash != expected_hash {
        return Err(ClientError::HashValidationError());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_mock::MockClient;
    use std::error::Error;
    use url::Url;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[tokio::test]
    async fn test_receive_secret_missing_key_fragment() -> Result<()> {
        let mock_client = MockClient::new();
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let url = Url::parse("https://example.com/secret/abc123")?;

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::Custom(msg)) if msg == "No key in URL"));
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_base64_key() -> Result<()> {
        let mock_client = MockClient::new().with_receive_success(b"some_data".to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        url.set_fragment(Some("invalid_base64!@#$"));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::Base64DecodeError(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_encrypted_data() -> Result<()> {
        let crypto_context = CryptoContext::generate();
        let key_base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(crypto_context.key());

        let mock_client = MockClient::new().with_receive_success(b"invalid_base64!@#$".to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        // Include hash in fragment: key:hash format
        let fragment = format!("{}:validhash123", key_base64);
        url.set_fragment(Some(&fragment));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::Base64DecodeError(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_payload_too_short() -> Result<()> {
        let crypto_context = CryptoContext::generate();
        let key_base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(crypto_context.key());

        // Create a payload that's too short (less than 12 bytes for nonce)
        let short_payload = vec![1, 2, 3, 4, 5];
        let encoded_data = base64::prelude::BASE64_STANDARD.encode(&short_payload);

        let mock_client = MockClient::new().with_receive_success(encoded_data.as_bytes().to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        // Include hash in fragment: key:hash format
        let fragment = format!("{}:validhash123", key_base64);
        url.set_fragment(Some(&fragment));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::CryptoError(msg)) if msg == "Payload too short"));
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_key_produces_32_bytes() -> Result<()> {
        let crypto_context = CryptoContext::generate();
        assert_eq!(crypto_context.key().len(), 32);

        // Test that keys are different each time
        let crypto_context2 = CryptoContext::generate();
        assert_ne!(crypto_context.key(), crypto_context2.key());
        Ok(())
    }

    #[tokio::test]
    async fn test_append_key_to_link() -> Result<()> {
        let url = Url::parse("https://example.com/secret/abc123")?;
        let crypto_context = CryptoContext::generate();

        let key = crypto_context.key();
        let result = append_to_link(url.clone(), &crypto_context, "xyz");

        assert!(
            result
                .fragment()
                .expect("URL should have a fragment")
                .contains(&base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(key))
        );
        assert_eq!(result.host_str(), url.host_str());
        Ok(())
    }

    #[tokio::test]
    async fn test_end_to_end_encryption_decryption() -> Result<()> {
        let mock_client =
            MockClient::new().with_send_success(Url::parse("https://example.com/secret/test123")?);
        let crypto_client = CryptoClient::new(Box::new(mock_client.clone()));

        let base_url = Url::parse("https://example.com")?;
        let secret_data = b"This is a complete end-to-end test";
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        let payload = Payload::from_bytes(secret_data, None);

        // Send the secret
        let send_result = crypto_client
            .send_secret(base_url, payload, ttl, token, None)
            .await?;

        // Extract the encrypted data that was sent
        let encrypted_data = mock_client.get_sent_data().ok_or("No sent data")?;

        // Create a new mock client for receiving
        let mock_client_receive = MockClient::new().with_receive_success(encrypted_data);
        let crypto_client_receive = CryptoClient::new(Box::new(mock_client_receive));

        // Receive the secret using the URL with key
        let receive_result = crypto_client_receive
            .receive_secret(send_result, None)
            .await?;

        assert_eq!(receive_result.decode_bytes()?, secret_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_aes_gcm_data() -> Result<()> {
        let crypto_context = CryptoContext::generate();
        let key_base64 = crypto_context.key_as_base64();

        // Create a valid base64 payload but with invalid AES-GCM data
        let invalid_aes_data = vec![0u8; 16]; // 16 bytes: 12 for nonce + 4 for invalid ciphertext
        let encoded_data = base64::prelude::BASE64_STANDARD.encode(&invalid_aes_data);

        let mock_client = MockClient::new().with_receive_success(encoded_data.as_bytes().to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        // Include hash in fragment: key:hash format
        let fragment = format!("{}:validhash123", key_base64);
        url.set_fragment(Some(&fragment));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(
            matches!(result, Err(ClientError::CryptoError(msg)) if msg.contains("AES-GCM error"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_with_invalid_hash() -> Result<()> {
        // First, create a valid encrypted secret
        let mock_client =
            MockClient::new().with_send_success(Url::parse("https://example.com/secret/test123")?);
        let crypto_client = CryptoClient::new(Box::new(mock_client.clone()));

        let base_url = Url::parse("https://example.com")?;
        let payload = Payload {
            data: "Test secret with hash".to_string(),
            filename: None,
        };
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        // Send the secret to get encrypted data and URL with hash
        let send_result = crypto_client
            .send_secret(base_url, payload, ttl, token, None)
            .await?;

        // Extract the encrypted data
        let encrypted_data = mock_client.get_sent_data().ok_or("No sent data")?;

        // Modify the URL to have an invalid hash
        let mut modified_url = send_result.clone();
        let fragment_parts: Vec<&str> = send_result
            .fragment()
            .ok_or("No fragment")?
            .split(':')
            .collect();

        // Keep the key but change the hash
        let modified_fragment = format!("{}:invalidhash123", fragment_parts[0]);
        modified_url.set_fragment(Some(&modified_fragment));

        // Try to receive with invalid hash
        let mock_client_receive = MockClient::new().with_receive_success(encrypted_data);
        let crypto_client_receive = CryptoClient::new(Box::new(mock_client_receive));

        let result = crypto_client_receive
            .receive_secret(modified_url, None)
            .await;

        assert!(matches!(result, Err(ClientError::HashValidationError())));
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_without_hash_fails() -> Result<()> {
        // Test that URLs without hash are rejected for security
        let mock_client =
            MockClient::new().with_send_success(Url::parse("https://example.com/secret/test123")?);
        let crypto_client = CryptoClient::new(Box::new(mock_client.clone()));

        let base_url = Url::parse("https://example.com")?;
        let payload = Payload {
            data: "Test secret without hash".to_string(),
            filename: None,
        };
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        // Send the secret
        let send_result = crypto_client
            .send_secret(base_url, payload.clone(), ttl, token, None)
            .await?;

        // Extract the encrypted data
        let encrypted_data = mock_client.get_sent_data().ok_or("No sent data")?;

        // Remove the hash from the URL (keep only the key)
        let mut url_without_hash = send_result.clone();
        let fragment_parts: Vec<&str> = send_result
            .fragment()
            .ok_or("No fragment")?
            .split(':')
            .collect();

        // Set fragment to only the key (no hash)
        url_without_hash.set_fragment(Some(fragment_parts[0]));

        // Receive without hash - should fail with missing hash error
        let mock_client_receive = MockClient::new().with_receive_success(encrypted_data);
        let crypto_client_receive = CryptoClient::new(Box::new(mock_client_receive));

        let result = crypto_client_receive
            .receive_secret(url_without_hash, None)
            .await;

        // Should fail due to missing hash
        assert!(matches!(result, Err(ClientError::Custom(msg)) if msg.contains("Missing hash")));
        Ok(())
    }
}
