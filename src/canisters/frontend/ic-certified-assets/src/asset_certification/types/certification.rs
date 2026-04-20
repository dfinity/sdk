use candid::CandidType;
use serde::{Deserialize, Serialize};

pub type AssetKey = String;

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CertificateExpression {
    pub expression: String,
    pub expression_hash: [u8; 32],
}
