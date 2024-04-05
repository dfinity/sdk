use thiserror::Error;
use ic_agent::identity::{Delegation, SignedDelegation};

#[derive(Error, Debug)]
pub enum DelegationError {
    // #[error("An error occurred while managing identities: {0}")]
    // IdentityManagerError(String),
    #[error("An error occurred while parsing the identity: {0}")]
    IdentityError(String),
}

#[derive(Debug, candid::Deserialize, serde::Serialize)]
pub struct IdentityDelegation {
    base_identity: String,
    delegations: Vec<SignedDelegation>,
}


#[derive(Debug, candid::Deserialize, serde::Serialize)]
pub struct JSONDelegation {
    expiration: String,
    pubkey: String,
}

#[derive(Debug, candid::Deserialize, serde::Serialize)]
pub struct SignedJSONDelegation {
   delegation: JSONDelegation,
    signature: String,
}

// signdJSONDelegation into SignedDelegation
impl SignedJSONDelegation {
    pub fn to_delegation(&self) -> Result<SignedDelegation, DelegationError> {
        // validate signature
        // convert from string to u64
        let expiration = u64::from_str_radix(&self.delegation.expiration, 16)
            .map_err(|err| DelegationError::IdentityError(err.to_string()))?;
        // check if expiration is a valid timestamp (not in the past, not too far in the future)

        let now_in_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| DelegationError::IdentityError(err.to_string()))?
            .as_nanos();

        if expiration < now_in_nanos as u64 {
            return Err(DelegationError::IdentityError("Invalid delegation. This delegation has expired. Please request a fresh delegation and try again".to_string()));
        }

        let pubkey = hex::decode(&self.delegation.pubkey)
            .map_err(|err| DelegationError::IdentityError(err.to_string()))?;

        let delegation = SignedDelegation {
            delegation: Delegation {
            expiration: expiration,
            pubkey: pubkey,
            targets: Option::None,
        },
            signature: hex::decode(&self.signature)
                .map_err(|err| DelegationError::IdentityError(err.to_string()))?,
        };
        Ok(delegation)
    }
}

#[derive(Debug, candid::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JSONDelegationChain {
    delegations: Vec<SignedJSONDelegation>,
    public_key: String,
}

impl JSONDelegationChain {
    pub fn to_identity_delegation(&self) -> Result<IdentityDelegation, DelegationError> {
        let base_identity = self.public_key.clone();
        let delegations = self.delegations.iter().map(|d| {
            // convert from string to u64
            let expiration = u64::from_str_radix(&d.delegation.expiration, 16)
                .map_err(|err| DelegationError::IdentityError(err.to_string()))?;
            // check if expiration is a valid timestamp (not in the past, not too far in the future)

            let now_in_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| DelegationError::IdentityError(err.to_string()))?
                .as_nanos();

            if expiration < now_in_nanos as u64 {
                return Err(DelegationError::IdentityError("Invalid delegation. This delegation has expired. Please request a fresh delegation and try again".to_string()));
            }

            let pubkey = hex::decode(&d.delegation.pubkey)
                .map_err(|err| DelegationError::IdentityError(err.to_string()))?;

            let delegation = SignedDelegation {
                delegation: Delegation {
                expiration: expiration,
                pubkey: pubkey,
                targets: Option::None,
            },
                signature: hex::decode(&d.signature)
                    .map_err(|err| DelegationError::IdentityError(err.to_string()))?,
            };
            Ok(delegation)
        }).collect::<Result<Vec<SignedDelegation>, DelegationError>>()?;

        Ok(IdentityDelegation {
            base_identity,
            delegations,
        })
    }
}

