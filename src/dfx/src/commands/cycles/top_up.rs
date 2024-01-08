use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::operations::cycles_ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::cycle_amount_parser;
use candid::Principal;
use clap::Parser;
use slog::warn;
use std::time::{SystemTime, UNIX_EPOCH};

/// Send cycles to a canister.
#[derive(Parser)]
pub struct TopUpOpts {
    /// Send cycles to this canister.
    to: String,

    /// The number of cycles to send.
    #[arg(value_parser = cycle_amount_parser)]
    amount: u128,

    /// Transfer cycles from this subaccount.
    #[arg(long)]
    from_subaccount: Option<Subaccount>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction deduplication, default is system time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: TopUpOpts) -> DfxResult {
    let agent = env.get_agent();

    let amount = opts.amount;

    fetch_root_key_if_needed(env).await?;

    let created_at_time = opts.created_at_time.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let to = get_canister_id(env, &opts.to)?;
    let from_subaccount = opts.from_subaccount.map(|x| x.0);
    todo!();

    // TODO: Support this functionality again.
    // let result = cycles_ledger::send(
    //     agent,
    //     env.get_logger(),
    //     to,
    //     amount,
    //     created_at_time,
    //     from_subaccount,
    // )
    // .await;
    // if result.is_err() && opts.created_at_time.is_none() {
    //     warn!(
    //         env.get_logger(),
    //         "If you retry this operation, use --created-at-time {}", created_at_time
    //     );
    // }
    // let block_index = result?;

    // println!("Transfer sent at block index {block_index}");

    Ok(())
}

fn get_canister_id(env: &dyn Environment, s: &str) -> DfxResult<Principal> {
    let principal = Principal::from_text(s).or_else(|_| {
        env.get_canister_id_store()
            .and_then(|canister_id_store| canister_id_store.get(s))
    })?;
    Ok(principal)
}
