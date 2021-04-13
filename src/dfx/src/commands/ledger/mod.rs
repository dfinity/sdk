use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;

use clap::Clap;
use tokio::runtime::Runtime;

mod account_id;
mod balance;
mod create_canister;
// mod topup;
mod transfer;

/// Ledger commands.
#[derive(Clap)]
#[clap(name("ledger"))]
pub struct LedgerOpts {
    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    AccountId(account_id::AccountIdOpts),
    Balance(balance::BalanceOpts),
    CreateCanister(create_canister::CreateCanisterOpts),
    // TopUp(topup::TopUpOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, opts: LedgerOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::AccountId(v) => account_id::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::CreateCanister(v) => create_canister::exec(&agent_env, v).await,
            // SubCommand::TopUp(v) => topup::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
        }
    })
}
