use async_trait::async_trait;
use uuid::Uuid;

/// Defines the asynchronous interface for a client that can send and receive secrets.
#[async_trait]
pub trait Client {
    /// Sends a secret to be stored.
    ///
    /// # Arguments
    ///
    /// * `data` - A string slice that holds the secret data to be sent.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(String)` containing the unique ID of the stored secret.
    /// - `Err(ClientError)` with an error message if the operation fails.
    async fn send_secret(&self, data: &str) -> Result<String, ClientError>;

    /// Retrieves a secret from the store using its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the secret to be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(String)` containing the secret data.
    /// - `Err(ClientError)` with an error message if the secret is not found or another error occurs.
    async fn receive_secret(&self, id: Uuid) -> Result<String, ClientError>;
}

/// Represents an error that can occur during client operations.
pub struct ClientError {
    message: String,
}

impl ClientError {
    /// Creates a new `ClientError` with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - A string slice that holds the error message.
    pub fn new(message: &str) -> Self {
        ClientError {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
