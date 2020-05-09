use crate::Blob;
use openssl::sha::sha256;
use serde::de::Error;
use serde::export::TryFrom;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const ID_SELF_AUTHENTICATING_LEN: usize = 33;
const ID_SELF_AUTHENTICATING_SUFFIX: u8 = 0x02;
const ID_ANONYMOUS_SUFFIX: u8 = 0x04;
const ID_ANONYMOUS_BYTES: &[u8] = &[ID_ANONYMOUS_SUFFIX];

/// A principal describes the security context of an identity, namely
/// any identity that can be authenticated along with a specific
/// role. In the case of the Internet Computer this maps currently to
/// the identities that can be authenticated by a canister.
///
/// Note a principal is not necessarily tied with a public key-pair,
/// yet we need at least a key-pair of a related principal to sign
/// requests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Principal(PrincipalInner);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrincipalInner {
    /// Defined as H(public_key) || 0x02.
    SelfAuthenticating(Vec<u8>),

    /// The anonymous Principal.
    Anonymous,
}

impl Principal {
    /// Right now we are enforcing a Twisted Edwards Curve 25519 point
    /// as the public key.
    pub fn self_authenticating(public_key: impl AsRef<[u8]>) -> Self {
        let mut bytes = Vec::with_capacity(ID_SELF_AUTHENTICATING_LEN);
        let hash = sha256(public_key.as_ref());
        bytes.extend(&hash);
        // Now add a suffix denoting the identifier as representing a
        // self-authenticating principal.
        bytes.push(ID_SELF_AUTHENTICATING_SUFFIX);
        Self(PrincipalInner::SelfAuthenticating(bytes))
    }

    pub fn anonymous() -> Self {
        Self(PrincipalInner::SelfAuthenticating(vec![
            ID_ANONYMOUS_SUFFIX,
        ]))
    }
}

impl TryFrom<Blob> for Principal {
    type Error = String;

    fn try_from(bytes: Blob) -> Result<Self, Self::Error> {
        Self::try_from(bytes.0.as_slice())
    }
}

impl TryFrom<&[u8]> for Principal {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let last_byte = bytes.last().ok_or_else(|| {
            "empty slice of bytes can not be parsed into an principal identifier".to_owned()
        })?;
        match *last_byte {
            ID_SELF_AUTHENTICATING_SUFFIX => Ok(Principal(PrincipalInner::SelfAuthenticating(
                bytes.to_vec(),
            ))),
            ID_ANONYMOUS_SUFFIX => Ok(Principal(PrincipalInner::Anonymous)),
            suffix => Err(format!("not supported principal type: {}", suffix)),
        }
    }
}

impl AsRef<[u8]> for PrincipalInner {
    fn as_ref(&self) -> &[u8] {
        match self {
            PrincipalInner::SelfAuthenticating(v) => v,
            PrincipalInner::Anonymous => ID_ANONYMOUS_BYTES,
        }
    }
}

impl Serialize for Principal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for Principal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Principal, D::Error> {
        Principal::try_from(Blob::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_parsing() {
        let seed = [
            0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22,
            0x11, 0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44,
            0x33, 0x22, 0x11, 0x00,
        ];
        let principal: Principal = Principal::self_authenticating(&seed);
        assert_eq!(
            serde_cbor::from_slice::<Principal>(
                serde_cbor::to_vec(&principal)
                    .expect("Failed to serialize")
                    .as_slice()
            )
            .unwrap(),
            principal
        );
    }
}
