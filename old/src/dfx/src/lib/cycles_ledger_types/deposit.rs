use candid::CandidType;
use icrc_ledger_types::icrc1::account::Account;

#[derive(CandidType, Debug, Clone)]
pub struct DepositArg {
    pub to: Account,
    pub memo: Option<Vec<u8>>,
}
