//! A library for manipulating assets in an asset canister.
//!
//! # Example
//!
//! ```rust,no_run
//! use ic_agent::agent::{Agent, http_transport::ReqwestTransport};
//! use ic_agent::identity::BasicIdentity;
//! use ic_utils::Canister;
//! use std::time::Duration;
//! # async fn not_main() -> Result<(), Box<dyn std::error::Error>> {
//! # let replica_url = "";
//! # let pemfile = "";
//! # let canister_id = "";
//! let agent = Agent::builder()
//!     .with_transport(ReqwestTransport::create(replica_url)?)
//!     .with_identity(BasicIdentity::from_pem_file(pemfile)?)
//!     .build()?;
//! let canister = Canister::builder()
//!     .with_canister_id(canister_id)
//!     .with_agent(&agent)
//!     .build()?;
//! let logger = slog::Logger::root(slog::Discard, slog::o!());
//! ic_asset::sync(&canister, &[concat!(env!("CARGO_MANIFEST_DIR"), "assets/").as_ref()], &logger).await?;
//! # Ok(())
//! # }

#![deny(
    missing_docs,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

mod asset;
mod batch_upload;
mod canister_api;
pub mod error;
mod evidence;
mod sync;
mod upload;

pub use evidence::compute_evidence;
pub use sync::prepare_sync_for_proposal;
pub use sync::sync;
pub use upload::upload;
