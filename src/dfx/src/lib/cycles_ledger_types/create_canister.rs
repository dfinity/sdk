use candid::{CandidType, Nat, Principal};
use ic_utils::interfaces::management_canister::builders::CanisterSettings;
use serde::Deserialize;
use thiserror::Error;

#[derive(CandidType, Clone, Debug)]
pub struct CreateCanisterArgs {
    pub from_subaccount: Option<icrc_ledger_types::icrc1::account::Subaccount>,
    pub created_at_time: Option<u64>,
    pub amount: u128,
    pub creation_args: Option<CmcCreateCanisterArgs>,
}
#[derive(CandidType, Clone, Debug)]
pub struct CmcCreateCanisterArgs {
    pub subnet_selection: Option<SubnetSelection>,
    pub settings: Option<CanisterSettings>,
}
#[derive(CandidType, Clone, Debug)]
#[allow(dead_code)]
pub enum SubnetSelection {
    /// Choose a random subnet that satisfies the specified properties
    Filter(SubnetFilter),
    /// Choose a specific subnet
    Subnet { subnet: Principal },
}
#[derive(CandidType, Clone, Debug)]
pub struct SubnetFilter {
    pub subnet_type: Option<String>,
}
#[derive(CandidType, Clone, Debug, Deserialize, Error)]
pub enum CreateCanisterError {
    #[error("Insufficient funds. Current balance: {balance}")]
    InsufficientFunds { balance: u128 },
    #[error("Local clock too far behind.")]
    TooOld,
    #[error("Local clock too far ahead.")]
    CreatedInFuture { ledger_time: u64 },
    #[error("Cycles ledger temporarily unavailable.")]
    TemporarilyUnavailable,
    #[error("Duplicate of block {duplicate_of}.")]
    Duplicate {
        duplicate_of: Nat,
        canister_id: Option<Principal>,
    },
    #[error("Cycles ledger failed to create canister: {error}")]
    FailedToCreate {
        fee_block: Option<Nat>,
        refund_block: Option<Nat>,
        error: String,
    },
    #[error("Ledger error {error_code}: {message}")]
    GenericError { error_code: Nat, message: String },
}
#[derive(Deserialize, CandidType, Clone, Debug, PartialEq, Eq)]
pub struct CreateCanisterSuccess {
    pub block_id: Nat,
    pub canister_id: Principal,
}
