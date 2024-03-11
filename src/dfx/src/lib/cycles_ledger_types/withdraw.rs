// TODO: Support this functionality again.

// // Copied from https://github.com/dfinity/cycles-ledger/blob/main/cycles-ledger/src/endpoints.rs
// use candid::{CandidType, Nat, Principal};
// use ic_cdk::api::call::RejectionCode;
// // use icrc_ledger_types::icrc1::account::Subaccount;
// // use icrc_ledger_types::icrc1::transfer::BlockIndex;
// use serde::Deserialize;

// #[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct WithdrawArgs {
//     #[serde(default)]
//     pub from_subaccount: Option<Subaccount>,
//     pub to: Principal,
//     #[serde(default)]
//     pub created_at_time: Option<u64>,
//     pub amount: NumCycles,
// }

// #[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub enum WithdrawError {
//     BadFee {
//         expected_fee: NumCycles,
//     },
//     InsufficientFunds {
//         balance: NumCycles,
//     },
//     TooOld,
//     CreatedInFuture {
//         ledger_time: u64,
//     },
//     TemporarilyUnavailable,
//     Duplicate {
//         duplicate_of: BlockIndex,
//     },
//     FailedToWithdraw {
//         fee_block: Option<Nat>,
//         rejection_code: RejectionCode,
//         rejection_reason: String,
//     },
//     GenericError {
//         error_code: Nat,
//         message: String,
//     },
//     InvalidReceiver {
//         receiver: Principal,
//     },
// }
