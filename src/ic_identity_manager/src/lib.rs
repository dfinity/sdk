//! This crate is written with the purpose to provide seamless
//! authentication of all requests, as the IC Public Spec dictates, guide
//! any authorization actions and manage principals.  A user should be
//! able to use an IC agent tool, such as dfx, providing minimum input or
//! distraction.
//!
//! # Goals and Guidelines
//!
//! Users should not worry about the signature schemes used,
//! appropriate keys to be used, or authorizing devices.
//!
//! In summary, we want to ensure all requests performed by the agent
//! using this library provide seamless authrntication of every request
//! performed.
//!
//! We aim to keep the user happy while at the same time, authenticate
//! properly avoiding temporary measures. Namely, we should sign every
//! single request out of the box, no turn-off buttons. To that end,
//! we aim to offer a "works out of the box" experience, meanwhile at
//! appropriate intervals, as things stabilize, we expose more control
//! to the user. In the end we make the user happy out of the box,
//! while we still provide the means to the experienced user to
//! operate and experiment.
//!
//! Furthermore, we want to avoid teaching the user "bad habits". For
//! example, leaving unencrypted key PEM format files in a git
//! directory.
//!
//! As a result we need to work without requiring the user to provide
//! us key files in every invocation of dfx or other IC agent.
//!
//! We should not incentivize the user to provide their system
//! credentials to communicate with the IC either. Each principal will
//! have multiple associated keys that should be revocable and not
//! associated with any other service. Note that different canister
//! operations or communicating with different canisters may require
//! different principals. The user should not be forced to provide a
//! set of master credential on each invocation of the agent. Finally,
//! associating a system host with an IC request enables tracking and
//! makes portability an issue.
//!
//! As this is in development and constant improvement and features
//! are added we generally aim and advise to avoid exposing non-stable
//! features directly to the user. Internal representations are always
//! subject to change.
//!
//! To that end we do need to take special care for backwards
//! compatibility. We do not want a user to have issues while running
//! multiple projects, or migrating a project to a newer version of
//! the user agent and thus this library.
//!
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
