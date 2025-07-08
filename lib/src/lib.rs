//! Zero-knowledge secret sharing client library for Hakanai.
//!
//! This library provides a complete client implementation for sending and receiving
//! ephemeral secrets using the Hakanai service. All encryption and decryption
//! happens client-side, ensuring the server never has access to plaintext data.
//!
//! # Architecture
//!
//! The library uses a layered client architecture:
//! - `WebClient` - Handles HTTP communication
//! - `CryptoClient` - Adds AES-256-GCM encryption/decryption
//! - `SecretClient` - Handles `Payload` serialization/deserialization
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use hakanai_lib::{client, client::Client};
//! use std::time::Duration;
//! use url::Url;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with the default configuration
//! let client = client::new();
//!
//! // Send a text secret
//! let secret_url = client.send_secret(
//!     Url::parse("https://example.com")?,
//!     hakanai_lib::models::Payload {
//!         data: "My secret message".to_string(),
//!         filename: None,
//!     },
//!     Duration::from_secs(3600), // 1 hour TTL
//!     "auth-token".to_string(),
//!     None, // No custom options
//! ).await?;
//!
//! println!("Secret URL: {}", secret_url);
//!
//! // Receive the secret (normally done by recipient)
//! let payload = client.receive_secret(secret_url, None).await?;
//! println!("Retrieved: {}", payload.data);
//! # Ok(())
//! # }
//! ```
//!
//! ## Sending Binary Files
//!
//! ```no_run
//! use hakanai_lib::{client, client::Client, models::Payload};
//! use std::time::Duration;
//! use url::Url;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = client::new();
//!
//! // Read file and create payload
//! let file_contents = std::fs::read("document.pdf")?;
//! let payload = Payload::from_bytes(&file_contents, Some("document.pdf".to_string()));
//!
//! // Send the file
//! let secret_url = client.send_secret(
//!     Url::parse("https://example.com")?,
//!     payload,
//!     Duration::from_secs(86400), // 24 hour TTL
//!     "auth-token".to_string(),
//!     None,
//! ).await?;
//!
//! println!("File shared at: {}", secret_url);
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Progress Monitoring
//!
//! ```no_run
//! use hakanai_lib::{client, client::Client, models::Payload, options::SecretSendOptions};
//! use hakanai_lib::observer::DataTransferObserver;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use url::Url;
//!
//! struct ProgressReporter;
//!
//! #[async_trait::async_trait]
//! impl DataTransferObserver for ProgressReporter {
//!     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
//!         let percent = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
//!         println!("Upload progress: {:.1}%", percent);
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = client::new();
//!
//! let options = SecretSendOptions::new()
//!     .with_observer(Arc::new(ProgressReporter))
//!     .with_timeout(Duration::from_secs(300));
//!
//! let secret_url = client.send_secret(
//!     Url::parse("https://example.com")?,
//!     Payload { data: "Large secret data...".to_string(), filename: None },
//!     Duration::from_secs(3600),
//!     "auth-token".to_string(),
//!     Some(options),
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Complete Integration Example
//!
//! Building a file sharing application that demonstrates real-world usage:
//!
//! ```no_run
//! use hakanai_lib::{client, client::Client, models::Payload};
//! use hakanai_lib::options::SecretSendOptions;
//! use hakanai_lib::observer::DataTransferObserver;
//! use async_trait::async_trait;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use url::Url;
//!
//! // Progress tracker for file uploads
//! struct FileUploadTracker {
//!     filename: String,
//! }
//!
//! #[async_trait]
//! impl DataTransferObserver for FileUploadTracker {
//!     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
//!         let percentage = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
//!         println!("Uploading {}: {:.1}%", self.filename, percentage);
//!     }
//! }
//!
//! // Complete document sharing workflow
//! async fn share_document(
//!     server_url: &str,
//!     token: &str,
//!     file_path: &str,
//! ) -> Result<String, Box<dyn std::error::Error>> {
//!     // Read file from filesystem
//!     let file_data = std::fs::read(file_path)?;
//!     let filename = std::path::Path::new(file_path)
//!         .file_name()
//!         .and_then(|n| n.to_str())
//!         .unwrap_or("unknown")
//!         .to_string();
//!
//!     // Create payload with file metadata
//!     let payload = Payload::from_bytes(&file_data, Some(filename.clone()));
//!
//!     // Set up progress monitoring and timeouts
//!     let observer = Arc::new(FileUploadTracker { filename });
//!     let options = SecretSendOptions::new()
//!         .with_observer(observer)
//!         .with_timeout(Duration::from_secs(300))
//!         .with_chunk_size(8192);
//!
//!     // Send the file
//!     let client = client::new();
//!     let secret_url = client.send_secret(
//!         Url::parse(server_url)?,
//!         payload,
//!         Duration::from_secs(86400), // 24 hours
//!         token.to_string(),
//!         Some(options),
//!     ).await?;
//!
//!     Ok(secret_url.to_string())
//! }
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # Ok(())
//! # }
//! ```
//!
//! ## Testing Patterns
//!
//! Comprehensive testing strategies for applications using hakanai-lib:
//!
//! ```
//! use hakanai_lib::{client::{Client, ClientError}, models::Payload};
//! use hakanai_lib::options::{SecretSendOptions, SecretReceiveOptions};
//! use async_trait::async_trait;
//! use url::Url;
//! use std::time::Duration;
//! use std::collections::HashMap;
//! use std::sync::{Arc, Mutex};
//!
//! // Mock client for comprehensive testing
//! #[derive(Clone)]
//! pub struct TestClient {
//!     storage: Arc<Mutex<HashMap<String, String>>>,
//!     should_fail: Arc<Mutex<bool>>,
//!     delay_ms: Arc<Mutex<u64>>,
//! }
//!
//! impl TestClient {
//!     pub fn new() -> Self {
//!         Self {
//!             storage: Arc::new(Mutex::new(HashMap::new())),
//!             should_fail: Arc::new(Mutex::new(false)),
//!             delay_ms: Arc::new(Mutex::new(0)),
//!         }
//!     }
//!
//!     pub fn set_failure_mode(&self, should_fail: bool) {
//!         *self.should_fail.lock().unwrap() = should_fail;
//!     }
//!
//!     pub fn set_delay(&self, delay_ms: u64) {
//!         *self.delay_ms.lock().unwrap() = delay_ms;
//!     }
//!
//!     pub fn get_stored_count(&self) -> usize {
//!         self.storage.lock().unwrap().len()
//!     }
//! }
//!
//! #[async_trait]
//! impl Client<Payload> for TestClient {
//!     async fn send_secret(
//!         &self,
//!         _base_url: Url,
//!         payload: Payload,
//!         _ttl: Duration,
//!         _token: String,
//!         _opts: Option<SecretSendOptions>,
//!     ) -> Result<Url, ClientError> {
//!         // Simulate network delay
//!         let delay = *self.delay_ms.lock().unwrap();
//!         if delay > 0 {
//!             tokio::time::sleep(Duration::from_millis(delay)).await;
//!         }
//!
//!         // Simulate failures
//!         if *self.should_fail.lock().unwrap() {
//!             return Err(ClientError::Custom("Simulated network failure".to_string()));
//!         }
//!
//!         // Store the payload
//!         let id = uuid::Uuid::new_v4().to_string();
//!         let data = serde_json::to_string(&payload).unwrap();
//!         self.storage.lock().unwrap().insert(id.clone(), data);
//!         Ok(Url::parse(&format!("https://test.example.com/secret/{}", id)).unwrap())
//!     }
//!
//!     async fn receive_secret(
//!         &self,
//!         url: Url,
//!         _opts: Option<SecretReceiveOptions>,
//!     ) -> Result<Payload, ClientError> {
//!         let id = url.path().split('/').last().unwrap();
//!         let data = self.storage
//!             .lock()
//!             .unwrap()
//!             .remove(id) // One-time access like real service
//!             .ok_or_else(|| ClientError::Custom("Secret not found or already accessed".to_string()))?;
//!         
//!         let payload: Payload = serde_json::from_str(&data)
//!             .map_err(|e| ClientError::Json(e))?;
//!         Ok(payload)
//!     }
//! }
//!
//! // Example comprehensive test
//! # #[tokio::test]
//! # async fn test_comprehensive_workflow() -> Result<(), Box<dyn std::error::Error>> {
//! let client = TestClient::new();
//!     
//! // Test text payload
//! let text_payload = Payload {
//!     data: "test secret message".to_string(),
//!     filename: None,
//! };
//!
//! let url = client.send_secret(
//!     Url::parse("https://test.example.com")?,
//!     text_payload.clone(),
//!     Duration::from_secs(300),
//!     "test-token".to_string(),
//!     None,
//! ).await?;
//!
//! let received = client.receive_secret(url, None).await?;
//! assert_eq!(received.data, text_payload.data);
//!
//! // Test file payload
//! let file_data = b"PDF file content...";
//! let file_payload = Payload::from_bytes(file_data, Some("test.pdf".to_string()));
//!
//! let file_url = client.send_secret(
//!     Url::parse("https://test.example.com")?,
//!     file_payload.clone(),
//!     Duration::from_secs(300),
//!     "test-token".to_string(),
//!     None,
//! ).await?;
//!
//! let received_file = client.receive_secret(file_url, None).await?;
//! assert_eq!(received_file.filename, file_payload.filename);
//! assert_eq!(received_file.decode_bytes()?, file_data);
//!
//! // Test one-time access (should fail on second attempt)
//! let url2 = client.send_secret(
//!     Url::parse("https://test.example.com")?,
//!     text_payload,
//!     Duration::from_secs(300),
//!     "test-token".to_string(),
//!     None,
//! ).await?;
//!
//! client.receive_secret(url2.clone(), None).await?; // First access succeeds
//! let second_attempt = client.receive_secret(url2, None).await; // Second access fails
//! assert!(second_attempt.is_err());
//! # Ok(())
//! # }
//! ```
//!
//! ## Security Best Practices
//!
//! Security-focused implementation patterns for production applications:
//!
//! ```no_run
//! use hakanai_lib::{client, client::Client, models::Payload};
//! use std::time::Duration;
//! use url::Url;
//! use std::env;
//!
//! // Secure token management
//! fn get_secure_token() -> Result<String, Box<dyn std::error::Error>> {
//!     // Priority 1: Environment variable (for development)
//!     if let Ok(token) = env::var("HAKANAI_TOKEN") {
//!         if !token.is_empty() {
//!             return Ok(token);
//!         }
//!     }
//!
//!     // Priority 2: Secure file (for production)
//!     if let Ok(token_file) = env::var("HAKANAI_TOKEN_FILE") {
//!         let token = std::fs::read_to_string(token_file)?;
//!         let token = token.trim();
//!         if !token.is_empty() {
//!             return Ok(token.to_string());
//!         }
//!     }
//!
//!     Err("No authentication token found. Set HAKANAI_TOKEN or HAKANAI_TOKEN_FILE".into())
//! }
//!
//! // Input validation and sanitization
//! fn validate_secret_content(data: &str, max_size: usize) -> Result<(), String> {
//!     // Size validation
//!     if data.len() > max_size {
//!         return Err(format!("Secret too large: {} bytes (max: {})", data.len(), max_size));
//!     }
//!
//!     // Empty content check
//!     if data.trim().is_empty() {
//!         return Err("Secret cannot be empty".to_string());
//!     }
//!
//!     // Warn about potential sensitive patterns (don't block, just warn)
//!     let lower_data = data.to_lowercase();
//!     if lower_data.contains("password") || lower_data.contains("private_key") || lower_data.contains("secret_key") {
//!         eprintln!("Warning: Secret appears to contain sensitive keywords");
//!     }
//!
//!     Ok(())
//! }
//!
//! // Secure sharing with comprehensive validation
//! async fn secure_share_secret(
//!     data: &str,
//!     filename: Option<String>,
//!     ttl_hours: u32,
//! ) -> Result<String, Box<dyn std::error::Error>> {
//!     // Validate inputs
//!     validate_secret_content(data, 1024 * 1024)?; // 1MB limit
//!
//!     if let Some(ref name) = filename {
//!         if name.contains("..") || name.starts_with('/') {
//!             return Err("Invalid filename: path traversal detected".into());
//!         }
//!     }
//!
//!     // Security: Limit TTL to reasonable maximum
//!     let max_ttl_hours = 168; // 7 days
//!     let ttl_hours = ttl_hours.min(max_ttl_hours);
//!
//!     // Get authentication token securely
//!     let token = get_secure_token()?;
//!
//!     // Create payload
//!     let payload = Payload {
//!         data: data.to_string(),
//!         filename,
//!     };
//!
//!     // Send with validated TTL
//!     let client = client::new();
//!     let secret_url = client.send_secret(
//!         Url::parse("https://secrets.example.com")?,
//!         payload,
//!         Duration::from_secs(ttl_hours as u64 * 3600),
//!         token,
//!         None,
//!     ).await?;
//!
//!     // Security: Log the action (without exposing the content)
//!     use std::collections::hash_map::DefaultHasher;
//!     use std::hash::{Hash, Hasher};
//!     let mut hasher = DefaultHasher::new();
//!     secret_url.as_str().hash(&mut hasher);
//!     let url_hash = hasher.finish();
//!     println!("Secret shared - ID hash: {:x}, TTL: {}h", url_hash, ttl_hours);
//!
//!     Ok(secret_url.to_string())
//! }
//!
//! // Secure retrieval with audit logging
//! async fn secure_retrieve_secret(
//!     secret_url: &str,
//!     user_id: &str,
//! ) -> Result<Payload, Box<dyn std::error::Error>> {
//!     // Validate URL format
//!     let _url = Url::parse(secret_url)?;
//!
//!     // Log access attempt (hash the URL for privacy)
//!     use std::collections::hash_map::DefaultHasher;
//!     use std::hash::{Hash, Hasher};
//!     let mut hasher = DefaultHasher::new();
//!     secret_url.hash(&mut hasher);
//!     let url_hash = hasher.finish();
//!     println!("Access attempt - User: {}, URL hash: {:x}", user_id, url_hash);
//!
//!     let client = client::new();
//!     let url = Url::parse(secret_url)?;
//!     
//!     match client.receive_secret(url, None).await {
//!         Ok(payload) => {
//!             // Log successful access
//!             println!("Successful retrieval - User: {}, Size: {} bytes",
//!                      user_id, payload.data.len());
//!             Ok(payload)
//!         }
//!         Err(e) => {
//!             // Log access failure (important for security monitoring)
//!             eprintln!("Failed retrieval - User: {}, Error: {}", user_id, e);
//!             Err(e.into())
//!         }
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Example: Secure document sharing
//! let document_content = "Important business document content...";
//! let secret_url = secure_share_secret(
//!     document_content,
//!     Some("business_plan.txt".to_string()),
//!     24, // 24 hours
//! ).await?;
//!
//! println!("Document shared securely: {}", secret_url);
//!
//! // Example: Secure retrieval with audit trail
//! let retrieved = secure_retrieve_secret(&secret_url, "user123").await?;
//! println!("Retrieved document: {} bytes", retrieved.data.len());
//!
//! // Security: Clear sensitive data from memory
//! std::mem::drop(retrieved);
//! # Ok(())
//! # }
//! ```

pub mod client;
mod crypto;
pub mod models;
pub mod observer;
pub mod options;
mod web;
