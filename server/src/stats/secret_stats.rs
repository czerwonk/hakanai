use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Represents statistics related to a single secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretStats {
    /// Timestamp of when the secret was created (in seconds since UNIX epoch)
    pub created_at: u64,
    /// Time-to-live (TTL) of the secret in seconds
    pub ttl: u64,
    /// Timestamp of when the secret was retrieved, if it has been retrieved
    pub retrieved_at: Option<u64>,
}

impl SecretStats {
    /// Creates a new `SecretStats` instance with the current time as `created_at` and the specified TTL.
    pub fn new(ttl: u64) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            created_at,
            ttl,
            retrieved_at: None,
        }
    }

    /// Calculates the lifetime of the secret from creation to retrieval.
    pub fn lifetime(&self) -> Option<u64> {
        if let Some(retrieved) = self.retrieved_at {
            return Some(retrieved.saturating_sub(self.created_at));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_stats_lifetime_after_retrieved() {
        let stats = SecretStats {
            created_at: 100,
            ttl: 200,
            retrieved_at: Some(250),
        };

        assert_eq!(stats.lifetime(), Some(150));
    }

    #[test]
    fn test_secret_stats_lifetime_not_retrieved() {
        let stats_no_retrieved = SecretStats {
            created_at: 100,
            ttl: 200,
            retrieved_at: None,
        };

        assert_eq!(stats_no_retrieved.lifetime(), None);
    }

    #[test]
    fn test_new_sets_timestamp() {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let stats = SecretStats::new(300);
        assert!(stats.created_at >= current_time)
    }
}
