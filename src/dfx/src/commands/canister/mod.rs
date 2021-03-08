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
    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[clap(long)]
    network: Option<String>,

    /// Performs the call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[clap(long)]
    call_as_user: bool,

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
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let call_as_user = opts.call_as_user;
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Call(v) => call::exec(&agent_env, v).await,
            SubCommand::Create(v) => create::exec(&agent_env, v, call_as_user).await,
            SubCommand::Delete(v) => delete::exec(&agent_env, v, call_as_user).await,
            SubCommand::Id(v) => id::exec(&agent_env, v).await,
            SubCommand::Install(v) => install::exec(&agent_env, v, call_as_user).await,
            SubCommand::RequestStatus(v) => request_status::exec(&agent_env, v).await,
            SubCommand::SetController(v) => set_controller::exec(&agent_env, v, call_as_user).await,
            SubCommand::Start(v) => start::exec(&agent_env, v, call_as_user).await,
            SubCommand::Status(v) => status::exec(&agent_env, v, call_as_user).await,
            SubCommand::Stop(v) => stop::exec(&agent_env, v, call_as_user).await,
        }
    })
}
