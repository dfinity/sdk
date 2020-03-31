//! # Usage
//!
//! We expose a single type [`Identity`], currently providing only
//! signing and principal.
//!
//! # Examples
//! ```not_run
//! use ic_identity_manager::identity::Identity;
//!
//! let identity =
//! Identity::new(std::path::PathBuf::from("temp_dir")).expect("Failed to construct an identity object");
//! let _signed_message = identity.sign(b"Hello World! This is Bob").expect("Signing failed");
//! ```
//! # Identity Precedence
//! [TODO]
//!
//! # Providers
//!

/// Provides basic error type and messages.
pub mod crypto_error;
/// Defines an identity object and API.
pub mod identity;

mod basic;
mod types;

pub use identity::Identity;
