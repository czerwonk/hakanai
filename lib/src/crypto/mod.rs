// SPDX-License-Identifier: Apache-2.0
mod aes;
mod crypto_context;
#[cfg(test)]
mod mock;

use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use reqwest::Url;
use zeroize::{Zeroize, Zeroizing};

use crate::client::{Client, ClientError};
use crate::crypto::aes::AESCryptoContextFactory;
use crate::crypto::crypto_context::{CryptoContext, CryptoContextFactory};
use crate::models::Payload;
use crate::options::{SecretReceiveOptions, SecretSendOptions};
use crate::utils::hashing;

/// A [`Client<Payload>`] that wraps a transport client to add transparent
/// encryption and decryption of secrets.
///
/// On send, the payload is serialised, encrypted with a freshly generated key,
/// and a content integrity hash is computed. Both the key and the hash are
/// embedded in the URL fragment as `#key:hash` so that recipients can decrypt
/// and verify the secret without the server ever seeing the plaintext.
///
/// On receive, the key and hash are extracted from the URL fragment, the
/// ciphertext is decrypted, the hash is verified, and the payload is
/// deserialised before being returned to the caller.
///
/// **Note:** This is an internal implementation detail. Prefer `client::new()`
/// which returns a fully configured client.
///
/// # Examples
///
/// ```
/// use hakanai_lib::{client, client::Client, models::Payload};
/// use std::time::Duration;
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = client::new();
///
/// let secret_url = client.send_secret(
///     Url::parse("https://api.example.com")?,
///     Payload::from_bytes(b"My secret message"),
///     Duration::from_secs(3600),
///     "auth-token".to_string(),
///     None,
/// ).await?;
///
/// let received = client.receive_secret(secret_url, None).await?;
/// assert_eq!(received.data, b"My secret message");
/// # Ok(())
/// # }
/// ```
pub struct CryptoClient {
    inner_client: Box<dyn Client<Vec<u8>>>,
    factory: Box<dyn CryptoContextFactory>,
}

impl CryptoClient {
    /// Creates a new `CryptoClient` backed by AES-256-GCM encryption.
    pub fn new(inner_client: Box<dyn Client<Vec<u8>>>) -> Self {
        CryptoClient {
            inner_client,
            factory: Box::new(AESCryptoContextFactory),
        }
    }

    /// Creates a `CryptoClient` with a custom [`CryptoContextFactory`].
    #[allow(dead_code)]
    pub fn with_factory(
        inner_client: Box<dyn Client<Vec<u8>>>,
        factory: Box<dyn CryptoContextFactory>,
    ) -> Self {
        CryptoClient {
            inner_client,
            factory,
        }
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
        let mut crypto_context = self.factory.generate();

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

        let url = append_to_link(res, &*crypto_context, &hash);

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

        let crypto_context = self.factory.generate_from_key_base64(parts[0])?;
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

fn append_to_link(url: Url, crypto_context: &dyn CryptoContext, hash: &str) -> Url {
    let mut link = url.clone();

    let mut fragment = crypto_context.key_as_base64();
    fragment.push_str(&format!(":{hash}"));

    link.set_fragment(Some(&fragment));
    fragment.zeroize();

    link
}

fn decrypt(
    encoded_data: Vec<u8>,
    mut crypto_context: Box<dyn CryptoContext>,
    hash: String,
) -> Result<Payload, ClientError> {
    let payload = Zeroizing::new(base64::prelude::BASE64_STANDARD.decode(encoded_data)?);

    crypto_context.import_nonce(&payload)?;
    let nonce_size = crypto_context.nonce_size();
    let ciphertext = &payload[nonce_size..];
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
    use crate::crypto::mock::MockCryptoContextFactory;
    use base64::Engine;
    use std::error::Error;

    use url::Url;

    use crate::client_mock::MockClient;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    fn mock_key_base64() -> String {
        MockCryptoContextFactory.generate().key_as_base64()
    }

    fn mock_client() -> CryptoClient {
        CryptoClient::with_factory(
            Box::new(MockClient::<Vec<u8>>::new()),
            Box::new(MockCryptoContextFactory),
        )
    }

    fn mock_client_with_send_url(url: Url) -> (CryptoClient, MockClient<Vec<u8>>) {
        let mock_client = MockClient::new().with_send_success(url);
        let crypto_client = CryptoClient::with_factory(
            Box::new(mock_client.clone()),
            Box::new(MockCryptoContextFactory),
        );
        (crypto_client, mock_client)
    }

    fn mock_client_with_receive_data(data: Vec<u8>) -> CryptoClient {
        CryptoClient::with_factory(
            Box::new(MockClient::<Vec<u8>>::new().with_receive_success(data)),
            Box::new(MockCryptoContextFactory),
        )
    }

    #[tokio::test]
    async fn test_receive_secret_missing_key_fragment() -> Result<()> {
        let crypto_client = mock_client();
        let url = Url::parse("https://example.com/secret/abc123")?;

        let result = crypto_client.receive_secret(url, None).await;
        assert!(
            matches!(result, Err(ClientError::Custom(ref msg)) if msg == "No key in URL"),
            "Expected 'No key in URL', got: {:?}",
            result,
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_base64_key() -> Result<()> {
        let crypto_client = mock_client_with_receive_data(b"some_data".to_vec());

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        url.set_fragment(Some("invalid_base64!@#$:hash"));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(
            matches!(result, Err(ClientError::Base64DecodeError(_))),
            "Expected Base64DecodeError, got: {:?}",
            result,
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_invalid_encrypted_data() -> Result<()> {
        let crypto_client = mock_client_with_receive_data(b"invalid_base64!@#$".to_vec());

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        url.set_fragment(Some(&format!("{}:validhash", mock_key_base64())));

        let result = crypto_client.receive_secret(url, None).await;
        assert!(
            matches!(result, Err(ClientError::Base64DecodeError(_))),
            "Expected Base64DecodeError, got: {:?}",
            result,
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_payload_too_short() -> Result<()> {
        // Payload shorter than the mock nonce size (4 bytes)
        let short_payload = vec![1u8, 2, 3];
        let encoded = base64::prelude::BASE64_STANDARD.encode(&short_payload);

        let mut url = Url::parse("https://example.com/secret/abc123")?;
        url.set_fragment(Some(&format!("{}:validhash", mock_key_base64())));

        let result = mock_client_with_receive_data(encoded.into_bytes())
            .receive_secret(url, None)
            .await;

        assert!(
            matches!(result, Err(ClientError::CryptoError(ref msg)) if msg == "Payload too short"),
            "Expected 'Payload too short', got: {:?}",
            result,
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_decrypt_failure_propagates() -> Result<()> {
        // Send with the mock factory to obtain real encrypted data
        let (crypto_client, transport) =
            mock_client_with_send_url(Url::parse("https://example.com/secret/test123")?);
        let send_result = crypto_client
            .send_secret(
                Url::parse("https://example.com")?,
                Payload::from_bytes(b"secret"),
                Duration::from_secs(3600),
                "token".to_string(),
                None,
            )
            .await?;
        let encrypted_data = transport.get_sent_data().ok_or("No sent data")?;

        // Receive with a different (wrong) key — decrypt should fail
        let wrong_key = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(b"wrong_key");
        let hash = send_result
            .fragment()
            .ok_or("No fragment")?
            .split(':')
            .nth(1)
            .ok_or("No hash")?;
        let mut url = send_result.clone();
        url.set_fragment(Some(&format!("{}:{}", wrong_key, hash)));

        let result = mock_client_with_receive_data(encrypted_data)
            .receive_secret(url, None)
            .await;

        assert!(
            matches!(result, Err(ClientError::CryptoError(_))),
            "Expected CryptoError from wrong key, got: {:?}",
            result,
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_append_key_to_link() -> Result<()> {
        let url = Url::parse("https://example.com/secret/abc123")?;
        let ctx = MockCryptoContextFactory.generate();
        let key_b64 = ctx.key_as_base64();

        let result = append_to_link(url.clone(), &*ctx, "xyz");

        assert!(
            result
                .fragment()
                .expect("URL should have a fragment")
                .contains(&key_b64),
            "Fragment should contain the key",
        );
        assert_eq!(
            result.host_str(),
            url.host_str(),
            "Host should be unchanged"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_end_to_end_encryption_decryption() -> Result<()> {
        let (crypto_client, transport) =
            mock_client_with_send_url(Url::parse("https://example.com/secret/test123")?);

        let secret_data = b"This is a complete end-to-end test";
        let send_result = crypto_client
            .send_secret(
                Url::parse("https://example.com")?,
                Payload::from_bytes(secret_data),
                Duration::from_secs(3600),
                "test_token".to_string(),
                None,
            )
            .await?;

        let encrypted_data = transport.get_sent_data().ok_or("No sent data")?;
        let receive_result = mock_client_with_receive_data(encrypted_data)
            .receive_secret(send_result, None)
            .await?;

        assert_eq!(
            receive_result.data, secret_data,
            "Decrypted data must match original",
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_with_invalid_hash() -> Result<()> {
        let (crypto_client, transport) =
            mock_client_with_send_url(Url::parse("https://example.com/secret/test123")?);

        let send_result = crypto_client
            .send_secret(
                Url::parse("https://example.com")?,
                Payload::from_bytes(b"Test secret with hash"),
                Duration::from_secs(3600),
                "test_token".to_string(),
                None,
            )
            .await?;

        let encrypted_data = transport.get_sent_data().ok_or("No sent data")?;

        let key = send_result
            .fragment()
            .ok_or("No fragment")?
            .split(':')
            .next()
            .ok_or("No key")?;
        let mut tampered_url = send_result.clone();
        tampered_url.set_fragment(Some(&format!("{}:invalidhash", key)));

        let result = mock_client_with_receive_data(encrypted_data)
            .receive_secret(tampered_url, None)
            .await;

        assert!(
            matches!(result, Err(ClientError::HashValidationError())),
            "Expected HashValidationError, got: {:?}",
            result,
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_receive_secret_without_hash_fails() -> Result<()> {
        let (crypto_client, transport) =
            mock_client_with_send_url(Url::parse("https://example.com/secret/test123")?);

        let send_result = crypto_client
            .send_secret(
                Url::parse("https://example.com")?,
                Payload::from_bytes(b"Test secret without hash"),
                Duration::from_secs(3600),
                "test_token".to_string(),
                None,
            )
            .await?;

        let encrypted_data = transport.get_sent_data().ok_or("No sent data")?;

        let key = send_result
            .fragment()
            .ok_or("No fragment")?
            .split(':')
            .next()
            .ok_or("No key")?;
        let mut url_without_hash = send_result.clone();
        url_without_hash.set_fragment(Some(key));

        let result = mock_client_with_receive_data(encrypted_data)
            .receive_secret(url_without_hash, None)
            .await;

        assert!(
            matches!(result, Err(ClientError::Custom(ref msg)) if msg.contains("Missing hash")),
            "Expected missing hash error, got: {:?}",
            result,
        );
        Ok(())
    }
}
