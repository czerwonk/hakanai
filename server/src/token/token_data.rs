// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Token metadata stored in Redis.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TokenData {
    /// Optional upload size limit in bytes.
    pub upload_size_limit: Option<i64>,
}

impl TokenData {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(str)
    }
}
