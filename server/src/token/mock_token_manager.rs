use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use super::{TokenCreator, TokenData, TokenError, TokenValidator};

/// Mock implementation of TokenValidator and TokenCreator traits with builder pattern.
///
/// This mock allows configuring various test scenarios including:
/// - Valid user tokens with associated data
/// - Valid admin tokens
/// - Token creation success/failure
/// - Custom token generation results
#[derive(Clone)]
pub struct MockTokenManager {
    /// Valid user tokens mapped to their metadata
    user_tokens: Arc<Mutex<HashMap<String, TokenData>>>,
    /// Valid admin tokens
    admin_tokens: Arc<Mutex<Vec<String>>>,
    /// Whether token creation should fail
    creation_should_fail: Arc<Mutex<bool>>,
    /// Token to return on successful creation
    created_token: Arc<Mutex<String>>,
}

impl MockTokenManager {
    /// Create a new mock token manager
    pub fn new() -> Self {
        Self {
            user_tokens: Arc::new(Mutex::new(HashMap::new())),
            admin_tokens: Arc::new(Mutex::new(Vec::new())),
            creation_should_fail: Arc::new(Mutex::new(false)),
            created_token: Arc::new(Mutex::new("mock_token".to_string())),
        }
    }

    /// Add a valid user token with associated metadata
    pub fn with_user_token(self, token: &str, data: TokenData) -> Self {
        self.user_tokens
            .lock()
            .unwrap()
            .insert(token.to_string(), data);
        self
    }

    /// Add a valid admin token
    pub fn with_admin_token(self, token: &str) -> Self {
        self.admin_tokens.lock().unwrap().push(token.to_string());
        self
    }

    /// Configure token creation to fail
    pub fn with_creation_failure(self) -> Self {
        *self.creation_should_fail.lock().unwrap() = true;
        self
    }

    /// Configure the token to return on successful creation
    pub fn with_created_token(self, token: &str) -> Self {
        *self.created_token.lock().unwrap() = token.to_string();
        self
    }

    /// Add multiple user tokens with unlimited upload size
    pub fn with_unlimited_user_tokens(self, tokens: &[&str]) -> Self {
        let mut user_tokens = self.user_tokens.lock().unwrap();
        for token in tokens {
            user_tokens.insert(
                token.to_string(),
                TokenData {
                    upload_size_limit: None,
                },
            );
        }
        drop(user_tokens);
        self
    }

    /// Add user token with specific upload size limit
    pub fn with_limited_user_token(self, token: &str, size_limit: i64) -> Self {
        self.user_tokens.lock().unwrap().insert(
            token.to_string(),
            TokenData {
                upload_size_limit: Some(size_limit),
            },
        );
        self
    }

    /// Add multiple admin tokens
    pub fn with_admin_tokens(self, tokens: &[&str]) -> Self {
        let mut admin_tokens = self.admin_tokens.lock().unwrap();
        for token in tokens {
            admin_tokens.push(token.to_string());
        }
        drop(admin_tokens);
        self
    }
}

impl Default for MockTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenValidator for MockTokenManager {
    async fn validate_user_token(&self, token: &str) -> Result<TokenData, TokenError> {
        let user_tokens = self.user_tokens.lock().unwrap();
        if let Some(data) = user_tokens.get(token) {
            Ok(data.clone())
        } else {
            Err(TokenError::InvalidToken)
        }
    }

    async fn validate_admin_token(&self, token: &str) -> Result<(), TokenError> {
        let admin_tokens = self.admin_tokens.lock().unwrap();
        if admin_tokens.contains(&token.to_string()) {
            Ok(())
        } else {
            Err(TokenError::InvalidToken)
        }
    }
}

#[async_trait]
impl TokenCreator for MockTokenManager {
    async fn create_user_token(
        &self,
        _token_data: TokenData,
        _ttl: Duration,
    ) -> Result<String, TokenError> {
        if *self.creation_should_fail.lock().unwrap() {
            Err(TokenError::Custom("Mock creation failure".to_string()))
        } else {
            Ok(self.created_token.lock().unwrap().clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenData;

    #[tokio::test]
    async fn test_mock_token_manager_builder() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockTokenManager::new()
            .with_user_token(
                "user_token",
                TokenData {
                    upload_size_limit: Some(1024),
                },
            )
            .with_admin_token("admin_token")
            .with_created_token("new_token");

        // Test user token validation
        let result = mock.validate_user_token("user_token").await?;
        assert_eq!(result.upload_size_limit, Some(1024));

        // Test invalid user token
        let result = mock.validate_user_token("invalid_token").await;
        assert!(
            result.is_err(),
            "Expected error for invalid user token, got: {:?}",
            result
        );

        // Test admin token validation
        mock.validate_admin_token("admin_token").await?;

        // Test invalid admin token
        let result = mock.validate_admin_token("invalid_admin").await;
        assert!(
            result.is_err(),
            "Expected error for invalid admin token, got: {:?}",
            result
        );

        // Test token creation
        let result = mock
            .create_user_token(
                TokenData {
                    upload_size_limit: None,
                },
                Duration::from_secs(3600),
            )
            .await?;
        assert_eq!(result, "new_token");

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_token_manager_creation_failure() {
        let mock = MockTokenManager::new().with_creation_failure();

        let result = mock
            .create_user_token(
                TokenData {
                    upload_size_limit: None,
                },
                Duration::from_secs(3600),
            )
            .await;
        assert!(
            result.is_err(),
            "Expected error for token creation with failures, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_mock_token_manager_bulk_methods() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockTokenManager::new()
            .with_unlimited_user_tokens(&["token1", "token2"])
            .with_admin_tokens(&["admin1", "admin2"]);

        // Test multiple user tokens
        mock.validate_user_token("token1").await?;
        mock.validate_user_token("token2").await?;

        // Test multiple admin tokens
        mock.validate_admin_token("admin1").await?;
        mock.validate_admin_token("admin2").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_token_manager_limited_user_token() -> Result<(), Box<dyn std::error::Error>>
    {
        let mock = MockTokenManager::new().with_limited_user_token("limited_token", 2048);

        let result = mock.validate_user_token("limited_token").await?;
        assert_eq!(result.upload_size_limit, Some(2048));

        Ok(())
    }
}
