use crate::lib::error::DfxResult;
use crate::lib::operations::cycles_ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::cycle_amount_parser;
use crate::{lib::environment::Environment, util::clap::parsers::icrc_subaccount_parser};
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::warn;
use std::time::{SystemTime, UNIX_EPOCH};

/// Approves a principal to spend cycles on behalf of the approver.
#[derive(Parser)]
pub struct ApproveOpts {
    /// Allow this principal to spend cycles.
    spender: Principal,

    /// The number of cycles to approve.
    #[arg(value_parser = cycle_amount_parser)]
    amount: u128,

    /// Allow this subaccount to spend cycles.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    spender_subaccount: Option<Subaccount>,

    /// Approve cycles to be spent from this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    from_subaccount: Option<Subaccount>,

    /// The number of previously approved cycles.
    /// See https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-2/README.md for details.
    #[arg(long, value_parser = cycle_amount_parser)]
    expected_allowance: Option<u128>,

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

pub async fn exec(env: &dyn Environment, opts: ApproveOpts) -> DfxResult {
    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let created_at_time = opts.created_at_time.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let result = cycles_ledger::approve(
        agent,
        env.get_logger(),
        opts.amount,
        opts.spender,
        opts.spender_subaccount,
        opts.from_subaccount,
        opts.expected_allowance,
        opts.expires_at,
        created_at_time,
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

    println!("Approval sent at block index {block_index}");

    Ok(())
}
