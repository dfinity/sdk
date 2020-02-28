//! Provides identity management and operations for the
//! Internet Computer (IC). Namely, we generate, load and store
//! credentials related to principals, provide principal mapping
//! seamlessly with corresponding key-pairs.
//!
//! # Definitions
//! An identity is a construct that denotes the set of claims of an
//! entity about itself.
//!
//! A principal describes the security context of an identity, namely
//! any identity that can be authenticated along with a specific
//! role. In the case of the Internet Computer this maps currently to
//! the identities that can be authenticated by a canister.
//!
//! Identification is the procedure whereby an entity claims a certain
//! identity, while verification is the procedure whereby that claim
//! is checked. Authentication is the assertion of an entity’s claim
//! to an identity.
//!
//! A role represents the set of actions an entity equipped with that
//! role can exercise.
//!
//! An identifier is a sequence of bytes/string utilized as a name for
//! a principal. That allows a principal to be referenced.
//!
//! A controller is a principal with an administrative-control role
//! over a corresponding canister. Each canister has one or more
//! controllers. A controller can be a person, an organization, or
//! another canister

//!
//! # Examples
//! [TODO]
//! # Identity Precedence
//! [TODO]
//!
//! # Providers
//! [TODO]

pub mod crypto_error;
pub mod identity;
pub mod principal;
mod provider;
mod signature;
