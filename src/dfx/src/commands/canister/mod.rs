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
mod start;
mod status;
mod stop;
mod update_settings;

/// Manages canisters deployed on a network replica.
#[derive(Clap)]
#[clap(name("canister"))]
pub struct CanisterOpts {
    /// Override the compute network to connect to. By default, the local network is used.
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
    Start(start::CanisterStartOpts),
    Status(status::CanisterStatusOpts),
    Stop(stop::CanisterStopOpts),
    UpdateSettings(update_settings::UpdateSettingsOpts),
}

pub fn exec(env: &dyn Environment, opts: CanisterOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Call(v) => call::exec(&agent_env, v).await,
            SubCommand::Create(v) => create::exec(&agent_env, v).await,
            SubCommand::Delete(v) => delete::exec(&agent_env, v).await,
            SubCommand::Id(v) => id::exec(&agent_env, v).await,
            SubCommand::Install(v) => install::exec(&agent_env, v).await,
            SubCommand::RequestStatus(v) => request_status::exec(&agent_env, v).await,
            SubCommand::Start(v) => start::exec(&agent_env, v).await,
            SubCommand::Status(v) => status::exec(&agent_env, v).await,
            SubCommand::Stop(v) => stop::exec(&agent_env, v).await,
            SubCommand::UpdateSettings(v) => update_settings::exec(&agent_env, v).await,
        }
    })
}
