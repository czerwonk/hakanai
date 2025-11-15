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
