use crate::{Blob, PrincipalIdError};
use openssl::sha::Sha256;
use serde::{Serialize, Serializer};

/// Type alias for a sha256 result (ie. a u256).
type Sha256Hash = [u8; 32];

#[derive(Clone, Debug)]
enum InnerPrincipalId {
    OpaqueId(Vec<u8>),
    SelfAuthenticating {
        principal: Sha256Hash,
    },
    DerivedId {
        principal: Sha256Hash,
        derivation: [u8; 8],
    },
}

/// A Principal ID in the Public Spec.
///
/// For now, only supports Self-Authenticating principals.
#[derive(Clone, Debug)]
pub struct PrincipalId {
    inner: InnerPrincipalId,
}

impl PrincipalId {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PrincipalIdError> {
        let inner = match bytes.last() {
            Some(0x01) => InnerPrincipalId::OpaqueId(Vec::from(&bytes[0..bytes.len() - 1])),
            Some(0x02) => {
                if bytes.len() < 33 {
                    return Err(PrincipalIdError::NotEnoughBytes);
                }

                let mut principal: [u8; 32] = [0; 32];
                principal.clone_from_slice(&bytes[0..32]);
                InnerPrincipalId::SelfAuthenticating { principal }
            }
            Some(0x03) => {
                if bytes.len() < 41 {
                    return Err(PrincipalIdError::NotEnoughBytes);
                }

                let mut principal: [u8; 32] = [0; 32];
                let mut derivation: [u8; 8] = [0; 8];
                principal.clone_from_slice(&bytes[0..32]);
                derivation.clone_from_slice(&bytes[0..8]);
                InnerPrincipalId::DerivedId {
                    principal,
                    derivation,
                }
            }
            Some(x) => return Err(PrincipalIdError::InvalidPrincipalIdType(*x)),
            None => return Err(PrincipalIdError::NotEnoughBytes),
        };

        Ok(PrincipalId { inner })
    }

    pub fn self_authenticating(key: Blob) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key.as_slice());
        let principal = hasher.finish();

        PrincipalId {
            inner: InnerPrincipalId::SelfAuthenticating { principal },
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self.inner {
            InnerPrincipalId::OpaqueId(ref bytes) => {
                result.extend_from_slice(&bytes);
                result.push(0x01);
            }
            InnerPrincipalId::SelfAuthenticating { principal } => {
                result.extend_from_slice(&principal);
                result.push(0x02);
            }
            InnerPrincipalId::DerivedId {
                principal,
                derivation,
            } => {
                result.extend_from_slice(&principal);
                result.extend_from_slice(&derivation);
                result.push(0x03);
            }
        }

        result
    }
}

impl Serialize for PrincipalId {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_vec())
    }
}
