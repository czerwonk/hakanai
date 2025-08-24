/// A custom error type for validation errors in the models module.
#[derive(Debug, thiserror::Error)]
#[error("Validation failed: {message}")]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
