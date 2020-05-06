//! A provider is responsible for signing functionality and generating
//! principals. A user may have multiple principal & credential
//! providers.

// use crate::crypto_error::Result;
// use crate::types::Signature;

// /// A provider corresponds to a combination of principal and
// /// credentials. It provides an IdentityWallet to allow
// /// a user to authenticate with a particular principal and service
// /// services. We do not necessarily keep in memory related
// /// credentials, but can reach out to a third-party service to provide
// /// us with the principal and the signing functionality.
// ///
// /// Signing operation and access to principals provided by
// /// IdentityWallet trait, need to be Sync, and perhaps in the
// /// background perform a setup and maintain an open connection. When
// /// constructing the identity value we do not want to impose that
// /// constraint, or open connections to all possible means of
// /// authentication, that are provided in the identity profile we
// /// loaded.
// pub trait Provider {
//     /// Setup the corresponding principal and return a value
//     /// that can provide signing.
//     fn provide(&self) -> Result<Box<dyn IdentityWallet>>;
// }

// /// Provide access to a signing functionality to authenticate with a
// /// particular principal. It is the responsibility of any
// /// implementations to take care of any state changes that might be
// /// required to perform signing.
// pub trait IdentityWallet: Sync {
//     /// Sign provided byte-sequence and authenticate as the indicated
//     /// principal.
//     fn sign(&self, msg: &[u8]) -> Result<Signature>;
//     /// Return associated principal.
//     fn principal(&self) -> Principal;
// }

pub mod basic;

pub use basic::BasicProvider;
