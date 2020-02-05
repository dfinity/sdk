//! A user may have multiple principal&credential providers.

use crate::crypto_error::Result;
use crate::principal::Principal;
use crate::signature::Signature;

pub trait Provider {
    fn provide(&self) -> Result<Box<dyn IdentityWallet>>;
}

pub trait IdentityWallet {
    fn sign(&self, msg: &[u8]) -> Result<Signature>;
    fn principal(&self) -> Principal;
}

// Keeping it simple and public right now.
pub mod basic;
