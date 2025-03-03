use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::icpts::ICPTs;
use crate::util::clap::parsers::icrc_subaccount_parser;
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1::account::Subaccount;

/// Approve a principal to spend ICP on behalf of the approver.
#[derive(Parser)]
pub struct ApproveOpts {
    /// Approve ICP to be spent from this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    from_subaccount: Option<Subaccount>,

    /// Allow this principal to spend ICP.
    spender: Principal,

    /// Allow this subaccount to spend ICP.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    spender_subaccount: Option<Subaccount>,

    /// The number of ICPs to approve.
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[arg(long)]
    amount: ICPTs,

    /// The number of previously approved ICPs.
    /// See https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-2/README.md for details.
    #[arg(long)]
    expected_allowance: Option<ICPTs>,

    /// Transaction fee, default is 10000 e8s.
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,

    /// Timestamp until which the approval is valid. None means that the approval is valid indefinitely.
    #[arg(long)]
    expires_at: Option<u64>,

    /// Memo.
    #[arg(long)]
    memo: Option<u64>,
}

pub async fn exec(_env: &dyn Environment, _opts: ApproveOpts) -> DfxResult {
    Ok(())
}
