use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::operations::cycles_ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::cycle_amount_parser;
use anyhow::Context;
use candid::Principal;
use clap::{ArgGroup, Parser};
use std::time::{SystemTime, UNIX_EPOCH};

/// Transfer cycles to another principal.
#[derive(Parser)]
#[clap(
group(ArgGroup::new("target").multiple(false).required(true)),
)]
pub struct TransferOpts {
    /// The amount of cycles to send.
    #[arg(value_parser = cycle_amount_parser)]
    amount: u128,

    /// Transfer cycles from this subaccount.
    #[arg(long)]
    from_subaccount: Option<Subaccount>,

    /// Transfer cycles to this principal.
    #[arg(long, group = "target")]
    to_owner: Option<Principal>,

    /// Transfer cycles to this subaccount.
    #[arg(long, requires("to_owner"))]
    to_subaccount: Option<Subaccount>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,

    /// Transfer fee.
    #[arg(long, value_parser = cycle_amount_parser)]
    fee: Option<u128>,

    /// Memo.
    #[arg(long)]
    memo: Option<u64>,

    /// Canister ID of the cycles ledger canister.
    /// If not specified, the default cycles ledger canister ID will be used.
    // todo: remove this.  See https://dfinity.atlassian.net/browse/SDK-1262
    #[arg(long)]
    cycles_ledger_canister_id: Principal,
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let agent = env.get_agent();

    let amount = opts.amount;

    fetch_root_key_if_needed(env).await?;

    let created_at_time = opts.created_at_time.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let block_index = if let Some(to_owner) = opts.to_owner {
        let from_subaccount = opts.from_subaccount.map(|x| x.0);
        let to_subaccount = opts.to_subaccount.map(|x| x.0);
        cycles_ledger::transfer(
            agent,
            amount,
            from_subaccount,
            to_owner,
            to_subaccount,
            created_at_time,
            opts.fee,
            opts.memo,
            opts.cycles_ledger_canister_id,
        )
        .await
        .with_context(|| {
            format!(
                "If you retry this operation, use --created-at-time {}",
                created_at_time
            )
        })?
    } else {
        unreachable!();
    };

    println!("Transfer sent at block index {block_index}");

    Ok(())
}
