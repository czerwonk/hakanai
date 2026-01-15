// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Represents the data payload of a secret, which can be either a text message
/// or a file with optional metadata.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Payload {
    /// The base64-encoded data of the secret.
    pub data: Vec<u8>,

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
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
            filename: None,
        }
    }

    /// Sets the filename for the payload, indicating that it represents a file.
    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
        self
    }

    pub fn serialize(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[test]
    fn test_payload_from_bytes() -> Result<()> {
        let bytes = b"Hello, world!";
        let payload = Payload::from_bytes(bytes).with_filename("greeting.txt");
        assert_eq!(payload.filename, Some("greeting.txt".to_string()));
        assert_eq!(payload.data, bytes.to_vec());
        Ok(())
    }

    #[test]
    fn test_payload_serialization_roundtrip() -> Result<()> {
        let payload = Payload::from_bytes(b"test data").with_filename("test.txt");

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, payload.data);
        assert_eq!(deserialized.filename, payload.filename);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_text_no_filename() -> Result<()> {
        let bytes = b"Hello, world!";
        let payload = Payload::from_bytes(bytes);

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data, payload.data);
        assert_eq!(deserialized.filename, None);
        Ok(())
    }

    #[test]
    fn test_payload_serialize_empty() -> Result<()> {
        let payload = Payload::from_bytes(b"");

        let serialized = payload.serialize()?;
        let deserialized = Payload::deserialize(&serialized)?;

        assert_eq!(deserialized.data.len(), 0);
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
            let bytes = b"test content";
            let payload = Payload::from_bytes(bytes).with_filename(filename);

            let serialized = payload.serialize()?;
            let deserialized = Payload::deserialize(&serialized)?;

            assert_eq!(deserialized.filename, Some(filename.to_string()));
        }
        Ok(())
    }

    #[test]
    fn test_payload_from_bytes_without_filename() -> Result<()> {
        let bytes = b"Secret message";
        let payload = Payload::from_bytes(bytes);

        assert!(payload.filename.is_none());
        assert_eq!(payload.data, bytes);
        Ok(())
    }

    #[test]
    fn test_payload_zeroize() {
        let mut payload = Payload::from_bytes(b"sensitive data").with_filename("secret.txt");

        payload.zeroize();

        assert_eq!(payload.data.len(), 0);
        assert_eq!(payload.filename, Some("".to_string()));
    }
}
