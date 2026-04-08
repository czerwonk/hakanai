// SPDX-License-Identifier: Apache-2.0
use crate::client::ClientError;

/// Abstraction over a symmetric authenticated encryption context.
///
/// Implementors manage a key and nonce and expose encrypt/decrypt operations
/// along with helpers for nonce serialisation used in the wire format.
pub trait CryptoContext: Send + Sync {
    /// Encrypts `plaintext` and returns the raw ciphertext (without nonce).
    fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, ClientError>;

    /// Decrypts `payload` (raw ciphertext without nonce) and returns the plaintext.
    fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>, ClientError>;

    /// Returns the encryption key encoded as URL-safe Base64 (no padding).
    fn key_as_base64(&self) -> String;

    /// Reads the nonce from the start of `payload`, storing it for subsequent
    /// [`decrypt`](CryptoContext::decrypt) calls.
    fn import_nonce(&mut self, payload: &[u8]) -> Result<(), ClientError>;

    /// Prepends the current nonce to `ciphertext`, producing the wire-format blob.
    fn prepend_nonce_to_ciphertext(&self, ciphertext: &[u8]) -> Vec<u8>;

    /// Returns the nonce length in bytes expected by this context.
    fn nonce_size(&self) -> usize;
}

/// Creates [`CryptoContext`] instances for a specific algorithm.
///
/// Implement this trait to plug a new encryption algorithm into [`CryptoClient`](crate::crypto::CryptoClient).
pub trait CryptoContextFactory: Send + Sync {
    /// Returns a new context with a freshly generated key and nonce.
    fn generate(&self) -> Box<dyn CryptoContext>;

    /// Restores a context from a URL-safe Base64-encoded key.
    fn generate_from_key_base64(&self, key: &str) -> Result<Box<dyn CryptoContext>, ClientError>;
}
