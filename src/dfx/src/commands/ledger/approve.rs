use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::MAINNET_LEDGER_CANISTER_ID;
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::operations::ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::icrc_subaccount_parser;
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::{info, warn};
use std::time::{SystemTime, UNIX_EPOCH};

/// Approve a principal to spend ICP on your behalf.
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

    /// Transaction fee, default is 0.00010000 ICP (10000 e8s).
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

    /// Canister ID of the ledger canister.
    #[arg(long)]
    ledger_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: ApproveOpts) -> DfxResult {
    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let canister_id = opts
        .ledger_canister_id
        .unwrap_or(MAINNET_LEDGER_CANISTER_ID);

    let created_at_time = opts.created_at_time.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let result = ledger::icrc2_approve(
        agent,
        env.get_logger(),
        &canister_id,
        opts.from_subaccount,
        opts.spender,
        opts.spender_subaccount,
        opts.amount,
        opts.expected_allowance,
        opts.fee,
        created_at_time,
        opts.expires_at,
        opts.memo,
    )
    .await;

    if result.is_err() && opts.created_at_time.is_none() {
        warn!(
            env.get_logger(),
            "If you retry this operation, use --created-at-time {}", created_at_time
        );
    }
    let block_index = result?;

    info!(
        env.get_logger(),
        "Approval sent at block index {}", block_index
    );

    Ok(())
}
