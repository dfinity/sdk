use crate::lib::error::DfxResult;
use crate::lib::ledger_types::MAINNET_LEDGER_CANISTER_ID;
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, operations::ledger};
use crate::util::clap::parsers::icrc_subaccount_parser;
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1::{self, account::Subaccount};
use std::time::{SystemTime, UNIX_EPOCH};

/// Transfer ICP from the approver princiapl to another principal.
#[derive(Parser)]
pub struct TransferFromOpts {
    /// Deduct allowance from this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    spender_subaccount: Option<Subaccount>,

    /// Transfer ICP from this principal.
    #[arg(long)]
    from: Principal,

    /// Transfer ICP from this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    from_subaccount: Option<Subaccount>,

    /// Transfer ICP to this principal.
    to: Principal,

    /// Transfer ICP to this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    to_subaccount: Option<Subaccount>,

    /// The number of ICPs to transfer.
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[arg(long)]
    amount: ICPTs,

    /// Transaction fee, default is 0.00010000 ICP (10000 e8s).
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,

    /// Memo.
    #[arg(long)]
    memo: Option<u64>,

    /// Canister ID of the ledger canister.
    #[arg(long)]
    ledger_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: TransferFromOpts) -> DfxResult {
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

    let from = icrc1::account::Account {
        owner: opts.from,
        subaccount: opts.from_subaccount,
    };
    let to = icrc1::account::Account {
        owner: opts.to,
        subaccount: opts.to_subaccount,
    };

    let result = ledger::transfer_from(
        agent,
        env.get_logger(),
        &canister_id,
        opts.spender_subaccount,
        from,
        to,
        opts.amount,
        opts.fee,
        created_at_time,
        opts.memo,
    )
    .await;

    if result.is_err() && opts.created_at_time.is_none() {
        slog::warn!(
            env.get_logger(),
            "If you retry this operation, use --created-at-time {}",
            created_at_time
        );
    }
    let block_index = result?;

    slog::info!(
        env.get_logger(),
        "Transfer sent at block index {}",
        block_index
    );

    Ok(())
}
