// SPDX-License-Identifier: Apache-2.0

//! Utility functions and helpers for the Hakanai library.
//!
//! This module provides various utility functions that support the core functionality
//! of the library but don't belong to any specific domain module. These utilities
//! are designed to be reusable across different parts of the codebase.
//!
//! # Submodules
//!
//! - [`content_analysis`] - Functions for analyzing content types (binary vs text detection)
//! - [`hashing`] - Functions for hashing data
//! - [`ip`] - Functions for parsing IP addresses and CIDR notation
//! - [`size_parser`] - Functions for parsing human-readable size strings
//! - [`timestamp`] - Functions for handling and formatting timestamps
//!

pub mod content_analysis;
pub mod hashing;
pub mod ip;
pub mod size_parser;
pub mod timestamp;
