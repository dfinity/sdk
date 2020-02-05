use openssl::sha::sha256;
use ring::signature::Ed25519KeyPair;
use ring::signature::KeyPair;
use serde::{Deserialize, Serialize};

const SELF_AUTHENTICATING_PRINCIPAL_LEN: usize = 33;

/// A principal describes the security context of an identity, namely
/// the role. In the case of the Internet Computer this maps currently
/// to the identifiers exposed to a canister.
///
/// Note a principal is not necessarily tied with a public key-pair,
/// yet we need at least a key-pair of a related principal to sign
/// requests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Principal(PrincipalInner);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum PrincipalInner {
    /// Defined as H(public_key) || 0x02.
    SelfAuthenticating(Vec<u8>),
}

impl Principal {
    // Right now we are enforcing a Twisted Edwards Curve 25519 point as the public key.
    pub fn new_self_authenticating(key_pair: &Ed25519KeyPair) -> Self {
        let mut bytes = Vec::with_capacity(SELF_AUTHENTICATING_PRINCIPAL_LEN);
        let public_key = key_pair.public_key();
        let hash = sha256(public_key.as_ref());
        hash.iter().for_each(|x| bytes.push(*x));
        bytes.push(0x02);
        Self(PrincipalInner::SelfAuthenticating(bytes))
    }
}
