use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use dfx_core::identity::CallSender;
use tokio::runtime::Runtime;

mod call;
mod create;
mod delete;
mod deposit_cycles;
mod id;
mod info;
mod install;
mod logs;
mod metadata;
mod request_status;
mod send;
mod sign;
mod start;
mod status;
mod stop;
mod uninstall_code;
mod update_settings;
mod url;

/// Manages canisters deployed on a network replica.
#[derive(Parser)]
#[command(name = "canister")]
pub struct CanisterOpts {
    #[command(flatten)]
    network: NetworkOpt,

    /// Specify a wallet canister id to perform the call.
    /// If none specified, defaults to use the selected Identity's wallet canister.
    #[arg(long, global = true)]
    wallet: Option<String>,

    #[command(subcommand)]
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
    Logs(logs::LogsOpts),
    Url(url::CanisterUrlOpts),
}

pub fn exec(env: &dyn Environment, opts: CanisterOpts) -> DfxResult {
    let agent_env;
    let env = if matches!(&opts.subcmd, SubCommand::Id(_)) {
        env
    } else {
        agent_env = create_agent_environment(env, opts.network.to_network_name())?;
        &agent_env
    };
    let runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(async {
        let call_sender = CallSender::from(&opts.wallet)
            .map_err(|e| anyhow!("Failed to determine call sender: {}", e))?;
        match opts.subcmd {
            SubCommand::Call(v) => call::exec(env, v, &call_sender).await,
            SubCommand::Create(v) => create::exec(env, v, &call_sender).await,
            SubCommand::Delete(v) => delete::exec(env, v, &call_sender).await,
            SubCommand::DepositCycles(v) => deposit_cycles::exec(env, v, &call_sender).await,
            SubCommand::Id(v) => id::exec(env, v).await,
            SubCommand::Install(v) => install::exec(env, v, &call_sender).await,
            SubCommand::Info(v) => info::exec(env, v).await,
            SubCommand::Metadata(v) => metadata::exec(env, v).await,
            SubCommand::RequestStatus(v) => request_status::exec(env, v).await,
            SubCommand::Send(v) => send::exec(env, v, &call_sender).await,
            SubCommand::Sign(v) => sign::exec(env, v, &call_sender).await,
            SubCommand::Start(v) => start::exec(env, v, &call_sender).await,
            SubCommand::Status(v) => status::exec(env, v, &call_sender).await,
            SubCommand::Stop(v) => stop::exec(env, v, &call_sender).await,
            SubCommand::UninstallCode(v) => uninstall_code::exec(env, v, &call_sender).await,
            SubCommand::UpdateSettings(v) => update_settings::exec(env, v, &call_sender).await,
            SubCommand::Logs(v) => logs::exec(env, v, &call_sender).await,
            SubCommand::Url(v) => url::exec(env, v).await,
        }
    })
}
