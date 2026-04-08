// SPDX-License-Identifier: Apache-2.0
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine;
use zeroize::{Zeroize, Zeroizing};

use crate::client::ClientError;
use crate::crypto::crypto_context::{CryptoContext, CryptoContextFactory};

const AES_GCM_KEY_SIZE: usize = 32; // AES-256 requires a 32-byte key
const AES_GCM_NONCE_SIZE: usize = 12; // AES-GCM uses a 12-byte nonce

/// AESCryptoContext is the AES256-GCM implementation for CryptoContext (which was used prior post quantum crypto)
pub struct AESCryptoContext {
    key: Vec<u8>,
    nonce: Vec<u8>,
    used: bool,
}

impl AESCryptoContext {
    pub fn generate() -> Self {
        let mut key = Zeroizing::new([0u8; AES_GCM_KEY_SIZE]);
        OsRng.fill_bytes(key.as_mut_slice());

        let mut nonce = Zeroizing::new([0u8; AES_GCM_NONCE_SIZE]);
        OsRng.fill_bytes(nonce.as_mut_slice());

        AESCryptoContext {
            key: key.to_vec(),
            nonce: nonce.to_vec(),
            used: false,
        }
    }

    pub fn from_key_base64(fragment: &str) -> Result<Self, ClientError> {
        let key = Zeroizing::new(base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(fragment)?);
        if key.len() != AES_GCM_KEY_SIZE {
            return Err(ClientError::CryptoError("Invalid key length".to_string()));
        }

        Ok(Self {
            key: key.to_vec(),
            nonce: vec![0u8; AES_GCM_NONCE_SIZE],
            used: false,
        })
    }

    #[cfg(test)]
    pub fn key(&self) -> &[u8] {
        &self.key
    }
}

/// Factory that produces [`AESCryptoContext`] instances.
pub struct AESCryptoContextFactory;

impl CryptoContextFactory for AESCryptoContextFactory {
    fn generate(&self) -> Box<dyn CryptoContext> {
        Box::new(AESCryptoContext::generate())
    }

    fn generate_from_key_base64(&self, key: &str) -> Result<Box<dyn CryptoContext>, ClientError> {
        Ok(Box::new(AESCryptoContext::from_key_base64(key)?))
    }
}

impl CryptoContext for AESCryptoContext {
    fn key_as_base64(&self) -> String {
        base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&self.key)
    }

    fn import_nonce(&mut self, payload: &[u8]) -> Result<(), ClientError> {
        if payload.len() < AES_GCM_NONCE_SIZE {
            return Err(ClientError::CryptoError("Payload too short".to_string()));
        }

        self.nonce = payload[..AES_GCM_NONCE_SIZE].to_vec();
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

        let key: &Key<Aes256Gcm> = self.key.as_slice().into();
        let cipher = Aes256Gcm::new(key);

        let mut nonce = Nonce::default();
        nonce.copy_from_slice(&self.nonce);

        Ok(cipher.encrypt(&nonce, plaintext)?)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ClientError> {
        let key: &Key<Aes256Gcm> = self.key.as_slice().into();
        let cipher = Aes256Gcm::new(key);

        let mut nonce = Nonce::default();
        nonce.copy_from_slice(&self.nonce);

        Ok(cipher.decrypt(&nonce, ciphertext)?)
    }

    fn nonce_size(&self) -> usize {
        AES_GCM_NONCE_SIZE
    }
}

impl Zeroize for AESCryptoContext {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.nonce.zeroize();
        self.used = false;
    }
}

impl Drop for AESCryptoContext {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn test_generate_key_produces_32_bytes() {
        let ctx = AESCryptoContext::generate();
        assert_eq!(ctx.key().len(), 32, "AES-256 key must be 32 bytes");

        let ctx2 = AESCryptoContext::generate();
        assert_ne!(ctx.key(), ctx2.key(), "Each generated key must be unique");
    }

    #[test]
    fn test_decrypt_invalid_aes_gcm_data_returns_error() {
        // 12-byte nonce + 4 bytes of garbage — valid structure but wrong ciphertext
        let ctx = AESCryptoContext::generate();
        let bad_data = vec![0u8; 4];
        let result = ctx.decrypt(&bad_data);
        assert!(
            matches!(result, Err(ClientError::CryptoError(ref msg)) if msg.contains("AES-GCM error")),
            "Expected AES-GCM error, got: {:?}",
            result,
        );
    }

    #[test]
    fn test_from_key_base64_wrong_length_returns_error() {
        let short_key = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(b"tooshort");
        let result = AESCryptoContext::from_key_base64(&short_key);
        assert!(
            matches!(result, Err(ClientError::CryptoError(ref msg)) if msg.contains("Invalid key length")),
            "Expected invalid key length error",
        );
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut ctx = AESCryptoContext::generate();
        let plaintext = b"secret message for roundtrip test";

        let ciphertext = ctx.encrypt(plaintext).expect("encryption should succeed");
        let wire = ctx.prepend_nonce_to_ciphertext(&ciphertext);

        let mut ctx2 = AESCryptoContext::from_key_base64(&ctx.key_as_base64())
            .expect("key import should succeed");
        ctx2.import_nonce(&wire)
            .expect("nonce import should succeed");
        let recovered = ctx2
            .decrypt(&wire[ctx2.nonce_size()..])
            .expect("decryption should succeed");

        assert_eq!(
            recovered, plaintext,
            "Decrypted plaintext must match original"
        );
    }

    #[test]
    fn test_import_nonce_payload_too_short() {
        let mut ctx = AESCryptoContext::generate();
        let short_payload = vec![1u8, 2, 3, 4, 5]; // fewer than 12 bytes (AES-GCM nonce size)
        let result = ctx.import_nonce(&short_payload);
        assert!(
            matches!(result, Err(ClientError::CryptoError(ref msg)) if msg == "Payload too short"),
            "Expected 'Payload too short', got: {:?}",
            result,
        );
    }
}
