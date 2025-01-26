use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use crate::lib::nns_types::icpts::ICPTs;
use anyhow::anyhow;
use clap::Parser;
use fn_error_context::context;
use tokio::runtime::Runtime;

mod account_id;
mod balance;
pub mod create_canister;
mod fabricate_cycles;
mod notify;
pub mod show_subnet_types;
mod top_up;
mod transfer;

/// Ledger commands.
#[derive(Parser)]
#[command(name = "ledger")]
pub struct LedgerOpts {
    #[command(flatten)]
    network: NetworkOpt,

    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    AccountId(account_id::AccountIdOpts),
    Balance(balance::BalanceOpts),
    CreateCanister(create_canister::CreateCanisterOpts),
    FabricateCycles(fabricate_cycles::FabricateCyclesOpts),
    Notify(notify::NotifyOpts),
    ShowSubnetTypes(show_subnet_types::ShowSubnetTypesOpts),
    TopUp(top_up::TopUpOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, opts: LedgerOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.to_network_name())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::AccountId(v) => account_id::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::CreateCanister(v) => create_canister::exec(&agent_env, v).await,
            SubCommand::FabricateCycles(v) => fabricate_cycles::exec(&agent_env, v).await,
            SubCommand::Notify(v) => notify::exec(&agent_env, v).await,
            SubCommand::ShowSubnetTypes(v) => show_subnet_types::exec(&agent_env, v).await,
            SubCommand::TopUp(v) => top_up::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
        }
    })
}

#[context("Failed to determine icp amount from supplied arguments.")]
pub(crate) fn get_icpts_from_args(
    amount: Option<ICPTs>,
    icp: Option<u64>,
    e8s: Option<u64>,
) -> DfxResult<ICPTs> {
    match amount {
        None => {
            let icp = match icp {
                Some(icps) => ICPTs::from_icpts(icps).map_err(|err| anyhow!(err))?,
                None => ICPTs::from_e8s(0),
            };
            let icp_from_e8s = match e8s {
                Some(e8s) => ICPTs::from_e8s(e8s),
                None => ICPTs::from_e8s(0),
            };
            let amount = icp + icp_from_e8s;
            Ok(amount.map_err(|err| anyhow!(err))?)
        }
        Some(amount) => Ok(amount),
    }
}
