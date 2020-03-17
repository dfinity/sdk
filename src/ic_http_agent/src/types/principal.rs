use openssl::sha::sha256;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

const SELF_AUTHENTICATING_PRINCIPAL_LEN: usize = 33;

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
}

impl Principal {
    /// Right now we are enforcing a Twisted Edwards Curve 25519 point
    /// as the public key.
    pub fn self_authenticating(public_key: impl AsRef<[u8]>) -> Self {
        let mut bytes = Vec::with_capacity(SELF_AUTHENTICATING_PRINCIPAL_LEN);
        let hash = sha256(public_key.as_ref());
        bytes.extend(&hash);
        // Now add a suffix denoting the identifier as representing a
        // self-authenticating principal.
        bytes.push(0x02);
        Self(PrincipalInner::SelfAuthenticating(bytes))
    }
}

impl AsRef<[u8]> for Principal {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for PrincipalInner {
    fn as_ref(&self) -> &[u8] {
        match self {
            PrincipalInner::SelfAuthenticating(v) => v,
        }
    }
}

impl Serialize for Principal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0.clone() {
            PrincipalInner::SelfAuthenticating(item) => item.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Principal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Principal, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        let last_byte = bytes
            .last()
            .ok_or_else(|| {
                "empty slice of bytes can not be parsed into an principal identifier".to_owned()
            })
            .map_err(de::Error::custom)?;
        match last_byte {
            0x02 => Ok(Principal(PrincipalInner::SelfAuthenticating(bytes))),
            _ => {
                let err_str = "not supported".to_owned();
                Err(de::Error::custom(err_str))
            }
        }
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
