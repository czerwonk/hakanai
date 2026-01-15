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

    #[test]
    fn test_deserialize_invalid_msgpack() {
        let invalid_bytes = b"not valid msgpack data";
        let result = Payload::deserialize(invalid_bytes);
        assert!(result.is_err(), "should fail on invalid msgpack data");
    }

    #[test]
    fn test_deserialize_empty_bytes() {
        let result = Payload::deserialize(&[]);
        assert!(result.is_err(), "should fail on empty input");
    }

    #[test]
    fn test_deserialize_wrong_structure() {
        // Valid msgpack but wrong structure (a simple integer)
        let wrong_structure =
            rmp_serde::to_vec(&42i32).expect("failed to serialize wrong structure");
        let result = Payload::deserialize(&wrong_structure);
        assert!(result.is_err(), "should fail on wrong msgpack structure");
    }

    #[test]
    fn test_serialize_with_filename() {
        let payload = Payload::from_bytes(b"secret data").with_filename("document.pdf");

        let serialized = payload.serialize().expect("serialization should succeed");

        // MessagePack format: fixarray(2) + bin8(11) + "secret data" + fixstr(12) + "document.pdf"
        let expected: Vec<u8> = vec![
            146, // fixarray with 2 elements
            155, // bin8 marker (0x9b = fixstr would be wrong, this is actually fixstr len 11)
            115, 101, 99, 114, 101, 116, 32, 100, 97, 116, 97,  // "secret data"
            172, // fixstr with 12 chars
            100, 111, 99, 117, 109, 101, 110, 116, 46, 112, 100, 102, // "document.pdf"
        ];
        assert_eq!(
            serialized, expected,
            "serialized bytes should match expected msgpack format"
        );
    }

    #[test]
    fn test_serialize_without_filename() {
        let payload = Payload::from_bytes(b"text message");

        let serialized = payload.serialize().expect("serialization should succeed");

        // MessagePack format: fixarray(2) + bin8(12) + "text message" + nil
        let expected: Vec<u8> = vec![
            146, // fixarray with 2 elements
            156, // fixstr with 12 chars
            116, 101, 120, 116, 32, 109, 101, 115, 115, 97, 103, 101, // "text message"
            192, // nil (None for filename)
        ];
        assert_eq!(
            serialized, expected,
            "serialized bytes should match expected msgpack format"
        );
    }
}
