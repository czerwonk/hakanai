// SPDX-License-Identifier: Apache-2.0

use base64::Engine;

use crate::client::ClientError;
use crate::crypto::crypto_context::{CryptoContext, CryptoContextFactory};

const MOCK_NONCE_SIZE: usize = 4;
const FIXED_KEY: &[u8] = b"mock_key_for_test";

/// A deterministic [`CryptoContext`] for testing.
///
/// Encryption produces `base64(key || plaintext)`. Decryption verifies the key
/// prefix and returns the remainder, so using the wrong key yields a
/// [`ClientError::CryptoError`] without needing a separate failure flag.
/// A fixed-size nonce is prepended to the wire payload so the full nonce
/// lifecycle is exercised without coupling tests to a specific algorithm.
pub(super) struct MockCryptoContext {
    key: Vec<u8>,
    nonce: [u8; MOCK_NONCE_SIZE],
}

impl CryptoContext for MockCryptoContext {
    fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, ClientError> {
        let mut buf = self.key.clone();
        buf.extend_from_slice(plaintext);
        Ok(base64::prelude::BASE64_STANDARD.encode(&buf).into_bytes())
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ClientError> {
        let buf = base64::prelude::BASE64_STANDARD.decode(ciphertext)?;
        if buf.get(..self.key.len()) != Some(self.key.as_slice()) {
            return Err(ClientError::CryptoError("mock: wrong key".to_string()));
        }
        Ok(buf[self.key.len()..].to_vec())
    }

    fn key_as_base64(&self) -> String {
        base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&self.key)
    }

    fn import_nonce(&mut self, payload: &[u8]) -> Result<(), ClientError> {
        if payload.len() < MOCK_NONCE_SIZE {
            return Err(ClientError::CryptoError("Payload too short".to_string()));
        }
        self.nonce.copy_from_slice(&payload[..MOCK_NONCE_SIZE]);
        Ok(())
    }

    fn prepend_nonce_to_ciphertext(&self, ciphertext: &[u8]) -> Vec<u8> {
        let mut result = self.nonce.to_vec();
        result.extend_from_slice(ciphertext);
        result
    }

    fn nonce_size(&self) -> usize {
        MOCK_NONCE_SIZE
    }
}

/// Factory that produces [`MockCryptoContext`] instances.
pub(super) struct MockCryptoContextFactory;

impl CryptoContextFactory for MockCryptoContextFactory {
    fn generate(&self) -> Box<dyn CryptoContext> {
        Box::new(MockCryptoContext {
            key: FIXED_KEY.to_vec(),
            nonce: [0u8; MOCK_NONCE_SIZE],
        })
    }

    fn generate_from_key_base64(&self, key: &str) -> Result<Box<dyn CryptoContext>, ClientError> {
        let key = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(key)?;
        Ok(Box::new(MockCryptoContext {
            key,
            nonce: [0u8; MOCK_NONCE_SIZE],
        }))
    }
}
