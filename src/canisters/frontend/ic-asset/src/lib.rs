//! A library for manipulating assets in an asset canister.
//!
//! # Example
//!
//! ```rust,no_run
//! use ic_agent::agent::{Agent, http_transport::ReqwestHttpReplicaV2Transport};
//! use ic_agent::identity::BasicIdentity;
//! use ic_utils::Canister;
//! use std::time::Duration;
//! # async fn not_main() -> Result<(), Box<dyn std::error::Error>> {
//! # let replica_url = "";
//! # let pemfile = "";
//! # let canister_id = "";
//! let agent = Agent::builder()
//!     .with_transport(ReqwestHttpReplicaV2Transport::create(replica_url)?)
//!     .with_identity(BasicIdentity::from_pem_file(pemfile)?)
//!     .build()?;
//! let canister = Canister::builder()
//!     .with_canister_id(canister_id)
//!     .with_agent(&agent)
//!     .build()?;
//! ic_asset::sync(&canister, &[concat!(env!("CARGO_MANIFEST_DIR"), "assets/").as_ref()], Duration::from_secs(60)).await?;
//! # Ok(())
//! # }

#![deny(
    missing_docs,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

mod asset_canister;
mod asset_config;
mod content;
mod content_encoder;
mod convenience;
mod operations;
mod params;
mod plumbing;
mod retryable;
mod semaphores;
mod sync;
mod upload;

pub use sync::sync;
pub use upload::upload;
