use crate::error::identity::IdentityError;
use crate::error::identity::IdentityError::{UnsupportedKeyVersion, ValidatePemContentFailed};

use ic_agent::identity::BasicIdentity;
use ic_agent::identity::PemError;
use ic_agent::identity::Secp256k1Identity;

pub fn validate_pem_file(pem_content: &[u8]) -> Result<(), IdentityError> {
    let secp_res =
        Secp256k1Identity::from_pem(pem_content).map_err(|e| ValidatePemContentFailed(Box::new(e)));
    if let Err(e) = secp_res {
        let basic_identity_res = BasicIdentity::from_pem(pem_content);
        match basic_identity_res {
            Err(PemError::KeyRejected(rj)) if rj.description_() == "VersionNotSupported" => {
                return Err(UnsupportedKeyVersion());
            }
            Err(_) => return Err(e),
            _ => {}
        }
    }

    Ok(())
}
