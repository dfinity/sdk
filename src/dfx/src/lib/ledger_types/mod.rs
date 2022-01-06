// DISCLAIMER:
// Do not modify this file arbitrarily.
// The contents are borrowed from:
// https://gitlab.com/dfinity-lab/public/ic/-/blob/master/rs/rosetta-api/ledger_canister/src/lib.rs
// https://github.com/dfinity/cdk-rs/blob/main/src/ic-ledger-types/src/lib.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use candid::CandidType;
use ic_types::principal::Principal;
use crate::lib::nns_types::{Memo, BlockHeight, TimeStamp};
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::ICPTs;

/// Id of the ledger canister on the IC.
pub const MAINNET_LEDGER_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x01]);

pub type AccountIdBlob = [u8; 32];

/// Arguments for the `transfer` call.
#[derive(CandidType)]
pub struct TransferArgs {
    pub memo: Memo,
    pub amount: ICPTs,
    pub fee: ICPTs,
    pub from_subaccount: Option<Subaccount>,
    pub to: AccountIdBlob,
    pub created_at_time: Option<TimeStamp>,
}

/// Result of the `transfer` call.
pub type TransferResult = Result<BlockHeight, TransferError>;

/// Error of the `transfer` call.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TransferError {
    BadFee { expected_fee: ICPTs },
    InsufficientFunds { balance: ICPTs },
    TxTooOld { allowed_window_nanos: u64 },
    TxCreatedInFuture,
    TxDuplicate { duplicate_of: BlockHeight },
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFee { expected_fee } => {
                write!(f, "transaction fee should be {}", expected_fee)
            }
            Self::InsufficientFunds { balance } => {
                write!(
                    f,
                    "the debit account doesn't have enough funds to complete the transaction, current balance: {}",
                    balance
                )
            }
            Self::TxTooOld {
                allowed_window_nanos,
            } => write!(
                f,
                "transaction is older than {} seconds",
                allowed_window_nanos / 1_000_000_000
            ),
            Self::TxCreatedInFuture => write!(f, "transaction's created_at_time is in future"),
            Self::TxDuplicate { duplicate_of } => write!(
                f,
                "transaction is a duplicate of another transaction in block {}",
                duplicate_of
            ),
        }
    }
}
