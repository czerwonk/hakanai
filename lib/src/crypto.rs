use std::time::Duration;

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, AeadCore, OsRng};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};

use async_trait::async_trait;
use base64::Engine;
use reqwest::Url;
use zeroize::Zeroizing;

use crate::client::{Client, ClientError};
use crate::options::{SecretReceiveOptions, SecretSendOptions};

/// A client that wraps another `Client` to provide cryptographic functionalities.
///
/// This struct is responsible for encrypting data before sending and decrypting
/// it upon reception, ensuring that secrets are transmitted securely.
///
/// The `CryptoClient` uses AES-256-GCM for authenticated encryption and embeds
/// the encryption key in the URL fragment for secure sharing.
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
/// // The URL contains the encryption key in the fragment
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
impl Client<Vec<u8>> for CryptoClient {
    async fn send_secret(
        &self,
        base_url: Url,
        data: Vec<u8>,
        ttl: Duration,
        token: String,
        opts: Option<SecretSendOptions>,
    ) -> Result<Url, ClientError> {
        let key = Zeroizing::new(generate_key());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_ref()));

        let ciphertext = cipher.encrypt(&nonce, &*data)?;

        // Prepend nonce to ciphertext
        let mut payload = nonce.to_vec();
        payload.extend_from_slice(&ciphertext);

        let encoded_data = base64::prelude::BASE64_STANDARD
            .encode(&payload)
            .as_bytes()
            .to_vec();

        let res = self
            .inner_client
            .send_secret(base_url, encoded_data, ttl, token, opts)
            .await?;

        let url = append_key_to_link(res, &key);

        Ok(url)
    }

    async fn receive_secret(
        &self,
        url: Url,
        opts: Option<SecretReceiveOptions>,
    ) -> Result<Vec<u8>, ClientError> {
        let key_base64 = url
            .fragment()
            .ok_or(ClientError::Custom("No key in URL".to_string()))?
            .to_string();

        let encoded_data = self.inner_client.receive_secret(url, opts).await?;
        decrypt(encoded_data, key_base64)
    }
}

fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);

    key
}

fn append_key_to_link(url: Url, key: &[u8; 32]) -> Url {
    let key_base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(key);

    let mut link = url.clone();
    link.set_fragment(Some(&key_base64));

    link
}

fn decrypt(encoded_data: Vec<u8>, key_base64: String) -> Result<Vec<u8>, ClientError> {
    let key = Zeroizing::new(base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(key_base64)?);

    let payload = base64::prelude::BASE64_STANDARD.decode(encoded_data)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    let nonce_len = aes_gcm::Nonce::<<Aes256Gcm as AeadCore>::NonceSize>::default().len();
    if payload.len() < nonce_len {
        return Err(ClientError::DecryptionError(
            "Payload too short".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = payload.split_at(nonce_len);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = Zeroizing::new(cipher.decrypt(nonce, ciphertext)?);
    Ok(plaintext.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::Mutex;
    use url::Url;

    #[derive(Clone)]
    struct MockClient {
        sent_data: Arc<Mutex<Option<Vec<u8>>>>,
        response_url: Option<Url>,
        response_data: Option<Vec<u8>>,
        should_error: bool,
        error_type: String,
    }

    impl MockClient {
        fn new() -> Self {
            Self {
                sent_data: Arc::new(Mutex::new(None)),
                response_url: None,
                response_data: None,
                should_error: false,
                error_type: String::new(),
            }
        }

        fn with_response_url(mut self, url: Url) -> Self {
            self.response_url = Some(url);
            self
        }

        fn with_response_data(mut self, data: Vec<u8>) -> Self {
            self.response_data = Some(data);
            self
        }

        fn get_sent_data(&self) -> Option<Vec<u8>> {
            self.sent_data.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Client<Vec<u8>> for MockClient {
        async fn send_secret(
            &self,
            _base_url: Url,
            data: Vec<u8>,
            _ttl: Duration,
            _token: String,
            _opts: Option<SecretSendOptions>,
        ) -> Result<Url, ClientError> {
            *self.sent_data.lock().unwrap() = Some(data);

            if self.should_error {
                return Err(ClientError::Custom(self.error_type.clone()));
            }

            Ok(self
                .response_url
                .clone()
                .unwrap_or_else(|| Url::parse("https://example.com/secret/12345").unwrap()))
        }

        async fn receive_secret(
            &self,
            _url: Url,
            _opts: Option<SecretReceiveOptions>,
        ) -> Result<Vec<u8>, ClientError> {
            if self.should_error {
                return Err(ClientError::Custom(self.error_type.clone()));
            }

            Ok(self
                .response_data
                .clone()
                .unwrap_or_else(|| "encrypted_data".as_bytes().to_vec()))
        }
    }

    #[tokio::test]
    async fn test_receive_secret_missing_key_fragment() {
        let mock_client = MockClient::new();
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let url = Url::parse("https://example.com/secret/abc123").unwrap();

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::Custom(msg)) if msg == "No key in URL"));
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_base64_key() {
        let mock_client = MockClient::new().with_response_data(b"some_data".to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123").unwrap();
        url.set_fragment(Some("invalid_base64!@#$"));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::Base64DecodeError(_))));
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_encrypted_data() {
        let key = generate_key();
        let key_base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(key);

        let mock_client = MockClient::new().with_response_data(b"invalid_base64!@#$".to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123").unwrap();
        url.set_fragment(Some(&key_base64));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(matches!(result, Err(ClientError::Base64DecodeError(_))));
    }

    #[tokio::test]
    async fn test_receive_secret_payload_too_short() {
        let key = generate_key();
        let key_base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(key);

        // Create a payload that's too short (less than 12 bytes for nonce)
        let short_payload = vec![1, 2, 3, 4, 5];
        let encoded_data = base64::prelude::BASE64_STANDARD.encode(&short_payload);

        let mock_client = MockClient::new().with_response_data(encoded_data.as_bytes().to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123").unwrap();
        url.set_fragment(Some(&key_base64));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(
            matches!(result, Err(ClientError::DecryptionError(msg)) if msg == "Payload too short")
        );
    }

    #[tokio::test]
    async fn test_generate_key_produces_32_bytes() {
        let key = generate_key();
        assert_eq!(key.len(), 32);

        // Test that keys are different each time
        let key2 = generate_key();
        assert_ne!(key, key2);
    }

    #[tokio::test]
    async fn test_append_key_to_link() {
        let url = Url::parse("https://example.com/secret/abc123").unwrap();
        let key = [42u8; 32];

        let result = append_key_to_link(url.clone(), &key);

        assert!(
            result
                .fragment()
                .expect("URL should have a fragment")
                .contains(&base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(key))
        );
        assert_eq!(result.host_str(), url.host_str());
    }

    #[tokio::test]
    async fn test_end_to_end_encryption_decryption() {
        let mock_client = MockClient::new()
            .with_response_url(Url::parse("https://example.com/secret/test123").unwrap());
        let crypto_client = CryptoClient::new(Box::new(mock_client.clone()));

        let base_url = Url::parse("https://example.com").unwrap();
        let secret_data = b"This is a complete end-to-end test";
        let ttl = Duration::from_secs(3600);
        let token = "test_token".to_string();

        // Send the secret
        let send_result = crypto_client
            .send_secret(base_url, secret_data.to_vec(), ttl, token, None)
            .await
            .unwrap();

        // Extract the encrypted data that was sent
        let encrypted_data = mock_client.get_sent_data().unwrap();

        // Create a new mock client for receiving
        let mock_client_receive = MockClient::new().with_response_data(encrypted_data);
        let crypto_client_receive = CryptoClient::new(Box::new(mock_client_receive));

        // Receive the secret using the URL with key
        let receive_result = crypto_client_receive
            .receive_secret(send_result, None)
            .await
            .unwrap();

        assert_eq!(receive_result, secret_data);
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_aes_gcm_data() {
        let key = generate_key();
        let key_base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(key);

        // Create a valid base64 payload but with invalid AES-GCM data
        let invalid_aes_data = vec![0u8; 16]; // 16 bytes: 12 for nonce + 4 for invalid ciphertext
        let encoded_data = base64::prelude::BASE64_STANDARD.encode(&invalid_aes_data);

        let mock_client = MockClient::new().with_response_data(encoded_data.as_bytes().to_vec());
        let crypto_client = CryptoClient::new(Box::new(mock_client));

        let mut url = Url::parse("https://example.com/secret/abc123").unwrap();
        url.set_fragment(Some(&key_base64));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(
            matches!(result, Err(ClientError::CryptoError(msg)) if msg.contains("AES-GCM error"))
        );
    }
}
