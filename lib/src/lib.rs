// SPDX-License-Identifier: Apache-2.0

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
//!         println!("Progress: {:.1}%", (bytes_transferred as f64 / total_bytes as f64) * 100.0);
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = client::new();
//! let options = SecretSendOptions::new().with_observer(Arc::new(ProgressReporter));
//!
//! client.send_secret(
//!     Url::parse("https://example.com")?,
//!     Payload { data: "Secret data".to_string(), filename: None },
//!     Duration::from_secs(3600),
//!     "auth-token".to_string(),
//!     Some(options),
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//!
//!

pub mod client;
#[cfg(any(test, feature = "testing"))]
pub mod client_mock;
mod crypto;
pub mod hash;
pub mod models;
pub mod observer;
pub mod options;
pub mod timestamp;
pub mod utils;
mod web;
