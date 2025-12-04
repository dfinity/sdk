pub mod canister;
pub mod canister_migration;
pub mod cmc;
pub mod cycles_ledger;
pub mod ledger;

const ICRC1_BALANCE_OF_METHOD: &str = "icrc1_balance_of";
const ICRC1_TRANSFER_METHOD: &str = "icrc1_transfer";
const ICRC2_ALLOWANCE_METHOD: &str = "icrc2_allowance";
const ICRC2_APPROVE_METHOD: &str = "icrc2_approve";
const ICRC2_TRANSFER_FROM_METHOD: &str = "icrc2_transfer_from";
