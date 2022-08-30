use crate::init_env;

use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::call_sender;
use crate::lib::provider::create_agent_environment;

use clap::{Args, Parser, Subcommand};
use tokio::runtime::Runtime;

use super::NetworkOpts;

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

#[derive(Args)]
pub struct CanisterOpts<T: Args> {
    #[clap(flatten)]
    network_opts: NetworkOpts<T>,

    /// Specify a wallet canister id to perform the call.
    /// If none specified, defaults to use the selected Identity's wallet canister.
    #[clap(long)]
    wallet: Option<String>,
}

/// Manages canisters deployed on a network replica.
#[derive(Parser)]
#[clap(name("canister"))]
pub struct CanisterCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    Call(CanisterOpts<call::CanisterCallOpts>),
    Create(CanisterOpts<create::CanisterCreateOpts>),
    Delete(CanisterOpts<delete::CanisterDeleteOpts>),
    DepositCycles(CanisterOpts<deposit_cycles::DepositCyclesOpts>),
    Id(CanisterOpts<id::CanisterIdOpts>),
    Info(CanisterOpts<info::InfoOpts>),
    Install(CanisterOpts<install::CanisterInstallOpts>),
    Metadata(CanisterOpts<metadata::CanisterMetadataOpts>),
    RequestStatus(CanisterOpts<request_status::RequestStatusOpts>),
    Send(CanisterOpts<send::CanisterSendOpts>),
    Sign(CanisterOpts<sign::CanisterSignOpts>),
    Start(CanisterOpts<start::CanisterStartOpts>),
    Status(CanisterOpts<status::CanisterStatusOpts>),
    Stop(CanisterOpts<stop::CanisterStopOpts>),
    UninstallCode(CanisterOpts<uninstall_code::UninstallCodeOpts>),
    UpdateSettings(CanisterOpts<update_settings::UpdateSettingsOpts>),
}

macro_rules! with_env {
    ($opts:expr, |$env:pat, $v:pat, $call_sender:pat_param| $e:expr) => {{
        let CanisterOpts {
            network_opts: NetworkOpts { base_opts, network },
            wallet,
        } = $opts;
        let env = init_env(base_opts.env_opts)?;
        let agent_env = create_agent_environment(&env, network)?;
        let $v = base_opts.command_opts;
        let runtime = Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(async {
            let $call_sender = call_sender(&agent_env, &wallet).await?;
            let $env = agent_env;
            $e.await?;
            Ok(())
        })
    }};
}

pub fn dispatch(cmd: CanisterCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Call(v) => {
            with_env!(v, |env, v, call_sender| call::exec(&env, v, &call_sender))
        }
        SubCommand::Create(v) => {
            with_env!(v, |env, v, call_sender| create::exec(&env, v, &call_sender))
        }
        SubCommand::Delete(v) => {
            with_env!(v, |env, v, call_sender| delete::exec(&env, v, &call_sender))
        }
        SubCommand::DepositCycles(v) => with_env!(v, |env, v, call_sender| {
            deposit_cycles::exec(&env, v, &call_sender)
        }),
        SubCommand::Id(v) => with_env!(v, |env, v, _| id::exec(&env, v)),
        SubCommand::Install(v) => with_env!(v, |env, v, call_sender| {
            install::exec(&env, v, &call_sender)
        }),
        SubCommand::Info(v) => with_env!(v, |env, v, _| info::exec(&env, v)),
        SubCommand::Metadata(v) => {
            with_env!(v, |env, v, _| metadata::exec(&env, v))
        }
        SubCommand::RequestStatus(v) => {
            with_env!(v, |env, v, _| request_status::exec(&env, v))
        }
        SubCommand::Send(v) => {
            with_env!(v, |env, v, call_sender| send::exec(&env, v, &call_sender))
        }
        SubCommand::Sign(v) => {
            with_env!(v, |env, v, call_sender| sign::exec(&env, v, &call_sender))
        }
        SubCommand::Start(v) => {
            with_env!(v, |env, v, call_sender| start::exec(&env, v, &call_sender))
        }
        SubCommand::Status(v) => {
            with_env!(v, |env, v, call_sender| status::exec(&env, v, &call_sender))
        }
        SubCommand::Stop(v) => {
            with_env!(v, |env, v, call_sender| stop::exec(&env, v, &call_sender))
        }
        SubCommand::UninstallCode(v) => with_env!(v, |env, v, call_sender| {
            uninstall_code::exec(&env, v, &call_sender)
        }),
        SubCommand::UpdateSettings(v) => with_env!(v, |env, v, call_sender| {
            update_settings::exec(&env, v, &call_sender)
        }),
    }
}
