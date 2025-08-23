// SPDX-License-Identifier: Apache-2.0

//! Data models and structures for the Hakanai library.
//!
//! This module contains all the data structures used throughout the Hakanai ecosystem,
//! including payload representations, API request/response models, access restrictions,
//! and token management structures.
//!
//! # Submodules
//!
//! - [`payload`] - Core payload structure for secrets (text/binary data with optional filename)
//! - [`restrictions`] - Access restriction models (IP-based filtering)
//! - [`secret`] - API request/response models for secret creation and retrieval
//! - [`token`] - Token management structures for admin API

pub mod payload;
pub mod restrictions;
pub mod secret;
pub mod token;

// Re-export all public types for convenience
pub use payload::Payload;
pub use restrictions::SecretRestrictions;
pub use secret::{PostSecretRequest, PostSecretResponse};
pub use token::{CreateTokenRequest, CreateTokenResponse};
