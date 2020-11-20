use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;

use clap::Clap;
use tokio::runtime::Runtime;

mod call;
mod create;
mod delete;
mod id;
mod install;
mod request_status;
mod set_controller;
mod start;
mod status;
mod stop;

/// Manages canisters deployed on a network replica.
#[derive(Clap)]
#[clap(name("canister"))]
pub struct CanisterOpts {
    // Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Call(call::CanisterCallOpts),
    Create(create::CanisterCreateOpts),
    Delete(delete::CanisterDeleteOpts),
    Id(id::CanisterIdOpts),
    Install(install::CanisterInstallOpts),
    RequestStatus(request_status::RequestStatusOpts),
    SetController(set_controller::SetControllerOpts),
    Start(start::CanisterStartOpts),
    Status(status::CanisterStatusOpts),
    Stop(stop::CanisterStopOpts),
}

pub fn exec(env: &dyn Environment, opts: CanisterOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    match opts.subcmd {
        SubCommand::Call(v) => runtime.block_on(call::exec(&agent_env, v)),
        SubCommand::Create(v) => runtime.block_on(create::exec(&agent_env, v)),
        SubCommand::Delete(v) => runtime.block_on(delete::exec(&agent_env, v)),
        SubCommand::Id(v) => runtime.block_on(id::exec(&agent_env, v)),
        SubCommand::Install(v) => runtime.block_on(install::exec(&agent_env, v)),
        SubCommand::RequestStatus(v) => runtime.block_on(request_status::exec(&agent_env, v)),
        SubCommand::SetController(v) => runtime.block_on(set_controller::exec(&agent_env, v)),
        SubCommand::Start(v) => runtime.block_on(start::exec(&agent_env, v)),
        SubCommand::Status(v) => runtime.block_on(status::exec(&agent_env, v)),
        SubCommand::Stop(v) => runtime.block_on(stop::exec(&agent_env, v)),
    }
}
