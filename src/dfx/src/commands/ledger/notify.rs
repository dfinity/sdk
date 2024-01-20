use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::{Parser, Subcommand};

mod create_canister;
mod top_up;

/// Notify the ledger about a send transaction to the cycles minting canister.
/// This command should only be used if `dfx ledger create-canister` or `dfx ledger top-up`
/// successfully sent a message to the ledger, and a transaction was recorded at some block height, but
/// for some reason the subsequent notify failed.
#[derive(Parser)]
pub struct NotifyOpts {
    #[command(subcommand)]
    subcmd: Subcmd,
}

#[derive(Subcommand)]
pub enum Subcmd {
    CreateCanister(create_canister::NotifyCreateOpts),
    TopUp(top_up::NotifyTopUpOpts),
}

pub async fn exec(env: &dyn Environment, opts: NotifyOpts) -> DfxResult {
    match opts.subcmd {
        Subcmd::CreateCanister(opts) => create_canister::exec(env, opts).await,
        Subcmd::TopUp(opts) => top_up::exec(env, opts).await,
    }
}
