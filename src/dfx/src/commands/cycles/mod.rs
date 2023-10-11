use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use clap::Parser;
use tokio::runtime::Runtime;

mod balance;

/// Helper commands to manage the user's cycles.
#[derive(Parser)]
#[command(name = "wallet")]
pub struct CyclesOpts {
    #[command(flatten)]
    network: NetworkOpt,

    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Balance(balance::CyclesBalanceOpts),
}

pub fn exec(env: &dyn Environment, opts: CyclesOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.to_network_name())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
        }
    })
}
