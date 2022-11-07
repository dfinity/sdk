use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::call_sender;
use crate::lib::provider::create_agent_environment;
use crate::{lib::environment::Environment, NetworkOpt};

use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;

mod call;
mod create;
mod delete;
mod deposit_cycles;
mod id;
mod info;
mod install;
mod metadata;
mod request_status;
mod send;
mod sign;
mod start;
mod status;
mod stop;
mod uninstall_code;
mod update_settings;

/// Manages canisters deployed on a network replica.
#[derive(Parser)]
#[clap(name("canister"))]
pub struct CanisterOpts {
    #[clap(flatten)]
    network: NetworkOpt,

    /// Specify a wallet canister id to perform the call.
    /// If none specified, defaults to use the selected Identity's wallet canister.
    #[clap(long, global(true))]
    wallet: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Call(call::CanisterCallOpts),
    Create(create::CanisterCreateOpts),
    Delete(delete::CanisterDeleteOpts),
    DepositCycles(deposit_cycles::DepositCyclesOpts),
    Id(id::CanisterIdOpts),
    Info(info::InfoOpts),
    Install(install::CanisterInstallOpts),
    Metadata(metadata::CanisterMetadataOpts),
    RequestStatus(request_status::RequestStatusOpts),
    Send(send::CanisterSendOpts),
    Sign(sign::CanisterSignOpts),
    Start(start::CanisterStartOpts),
    Status(status::CanisterStatusOpts),
    Stop(stop::CanisterStopOpts),
    UninstallCode(uninstall_code::UninstallCodeOpts),
    UpdateSettings(update_settings::UpdateSettingsOpts),
}

pub fn exec(env: &dyn Environment, opts: CanisterOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(async {
        let call_sender = call_sender(&agent_env, &opts.wallet).await?;
        match opts.subcmd {
            SubCommand::Call(v) => call::exec(&agent_env, v, &call_sender).await,
            SubCommand::Create(v) => create::exec(&agent_env, v, &call_sender).await,
            SubCommand::Delete(v) => delete::exec(&agent_env, v, &call_sender).await,
            SubCommand::DepositCycles(v) => deposit_cycles::exec(&agent_env, v, &call_sender).await,
            SubCommand::Id(v) => id::exec(&agent_env, v).await,
            SubCommand::Install(v) => install::exec(&agent_env, v, &call_sender).await,
            SubCommand::Info(v) => info::exec(&agent_env, v).await,
            SubCommand::Metadata(v) => metadata::exec(&agent_env, v).await,
            SubCommand::RequestStatus(v) => request_status::exec(&agent_env, v).await,
            SubCommand::Send(v) => send::exec(&agent_env, v, &call_sender).await,
            SubCommand::Sign(v) => sign::exec(&agent_env, v, &call_sender).await,
            SubCommand::Start(v) => start::exec(&agent_env, v, &call_sender).await,
            SubCommand::Status(v) => status::exec(&agent_env, v, &call_sender).await,
            SubCommand::Stop(v) => stop::exec(&agent_env, v, &call_sender).await,
            SubCommand::UninstallCode(v) => uninstall_code::exec(&agent_env, v, &call_sender).await,
            SubCommand::UpdateSettings(v) => {
                update_settings::exec(&agent_env, v, &call_sender).await
            }
        }
    })
}
