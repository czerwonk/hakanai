// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Token metadata stored in Redis.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TokenData {
    /// Optional upload size limit in bytes.
    pub upload_size_limit: Option<i64>,

    /// Wether the token is one-time use.
    #[serde(default)]
    pub one_time: bool,
}

impl TokenData {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(str)
    }

    pub fn with_upload_size_limit(mut self, upload_size_limit: i64) -> Self {
        self.upload_size_limit = Some(upload_size_limit);
        self
    }
}

impl Default for TokenData {
    fn default() -> Self {
        Self {
            upload_size_limit: None,
            one_time: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde_json;
    use tokio;

    #[tokio::test]
    async fn test_token_data_serialization() -> Result<()> {
        let token_data = TokenData::default().with_upload_size_limit(1024);

        // Test serialization
        let serialized = serde_json::to_string(&token_data)?;
        assert!(serialized.contains("1024"));

        // Test deserialization
        let deserialized: TokenData = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.upload_size_limit, Some(1024));
        Ok(())
    }

    #[tokio::test]
    async fn test_token_data_none_upload_limit() -> Result<()> {
        let token_data = TokenData::default();

        let serialized = serde_json::to_string(&token_data)?;
        let deserialized: TokenData = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.upload_size_limit, None);
        Ok(())
    }
}
