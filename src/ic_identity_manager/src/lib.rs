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
/// Defines various types of Principals, how their identifiers are
/// represented, and required encodings, conforming with the design
/// and public spec.
pub mod principal;

mod basic;
mod types;

pub use identity::Identity;
