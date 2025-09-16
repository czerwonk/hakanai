// SPDX-License-Identifier: Apache-2.0

//! Data models and structures for the Hakanai library.
//!
//! This module contains all the data structures used throughout the Hakanai ecosystem,
//! including payload representations, API request/response models, access restrictions,
//! and token management structures.
//!
//! # Submodules
//!
//! - [`country_code`] - ISO 3166-1 alpha-2 country code validation and representation
//! - [`errors`] - Common validation error types for model data structures
//! - [`payload`] - Core payload structure for secrets (text/binary data with optional filename)
//! - [`restrictions`] - Access restriction models (IP-based and geo-location filtering)
//! - [`secret`] - API request/response models for secret creation and retrieval
//! - [`token`] - Token management structures for admin API

pub mod country_code;
pub mod errors;
pub mod payload;
pub mod restrictions;
pub mod secret;
pub mod token;

pub use country_code::CountryCode;
pub use errors::ValidationError;
pub use payload::{Payload, PayloadDataType};
pub use restrictions::SecretRestrictions;
pub use secret::{PostSecretRequest, PostSecretResponse};
pub use token::{CreateTokenRequest, CreateTokenResponse};
