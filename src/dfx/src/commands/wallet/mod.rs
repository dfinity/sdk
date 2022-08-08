use crate::init_env;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::Context;
use candid::utils::ArgumentDecoder;
use candid::CandidType;
use clap::Parser;
use fn_error_context::context;
use ic_utils::call::SyncCall;
use ic_utils::interfaces::WalletCanister;
use tokio::runtime::Runtime;

use super::NetworkOpts;

mod add_controller;
mod authorize;
mod balance;
mod controllers;
mod custodians;
mod deauthorize;
mod list_addresses;
mod name;
mod remove_controller;
mod send;
mod set_name;
mod upgrade;

/// Helper commands to manage the user's cycles wallet.
#[derive(Parser)]
#[clap(name("wallet"))]
pub struct WalletCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Addresses(NetworkOpts<list_addresses::AddressesOpts>),
    AddController(NetworkOpts<add_controller::AddControllerOpts>),
    Authorize(NetworkOpts<authorize::AuthorizeOpts>),
    Balance(NetworkOpts<balance::WalletBalanceOpts>),
    Controllers(NetworkOpts<controllers::ControllersOpts>),
    Custodians(NetworkOpts<custodians::CustodiansOpts>),
    Deauthorize(NetworkOpts<deauthorize::DeauthorizeOpts>),
    Name(NetworkOpts<name::NameOpts>),
    RemoveController(NetworkOpts<remove_controller::RemoveControllerOpts>),
    Send(NetworkOpts<send::SendOpts>),
    SetName(NetworkOpts<set_name::SetNameOpts>),
    Upgrade(NetworkOpts<upgrade::UpgradeOpts>),
}

macro_rules! with_env {
    ($opts:expr, |$env:ident, $v:ident| $e:expr) => {{
        let NetworkOpts { base_opts, network } = $opts;
        let env = init_env(base_opts.env_opts)?;
        let $env = create_agent_environment(&env, network)?;
        let runtime = Runtime::new().expect("Unable to create a runtime");
        let $v = base_opts.command_opts;
        runtime.block_on($e)
    }};
}

pub fn dispatch(cmd: WalletCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Addresses(v) => {
            with_env!(v, |env, v| list_addresses::exec(&env, v))
        }
        SubCommand::AddController(v) => {
            with_env!(v, |env, v| add_controller::exec(&env, v))
        }
        SubCommand::Authorize(v) => with_env!(v, |env, v| authorize::exec(&env, v)),
        SubCommand::Balance(v) => with_env!(v, |env, v| balance::exec(&env, v)),
        SubCommand::Controllers(v) => {
            with_env!(v, |env, v| controllers::exec(&env, v))
        }
        SubCommand::Custodians(v) => {
            with_env!(v, |env, v| custodians::exec(&env, v))
        }
        SubCommand::Deauthorize(v) => {
            with_env!(v, |env, v| deauthorize::exec(&env, v))
        }
        SubCommand::Name(v) => with_env!(v, |env, v| name::exec(&env, v)),
        SubCommand::RemoveController(v) => {
            with_env!(v, |env, v| remove_controller::exec(&env, v))
        }
        SubCommand::Send(v) => with_env!(v, |env, v| send::exec(&env, v)),
        SubCommand::SetName(v) => with_env!(v, |env, v| set_name::exec(&env, v)),
        SubCommand::Upgrade(v) => with_env!(v, |env, v| upgrade::exec(&env, v)),
    }
}

#[context("Failed to call query function '{}' on wallet.", method)]
async fn wallet_query<A, O>(env: &dyn Environment, method: &str, arg: A) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let identity_name = env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    // Network descriptor will always be set.
    let network = env.get_network_descriptor();
    let wallet =
        Identity::get_or_create_wallet_canister(env, network, &identity_name, false).await?;

    let out: O = wallet
        .query_(method)
        .with_arg(arg)
        .build()
        .call()
        .await
        .context("Query to wallet failed.")?;
    Ok(out)
}

#[context("Failed to call update function '{}' on wallet.", method)]
async fn wallet_update<A, O>(env: &dyn Environment, method: &str, arg: A) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let wallet = get_wallet(env).await?;
    let out: O = wallet
        .update_(method)
        .with_arg(arg)
        .build()
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;
    Ok(out)
}

#[context("Failed to setup wallet caller.")]
async fn get_wallet(env: &dyn Environment) -> DfxResult<WalletCanister<'_>> {
    let identity_name = env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    // Network descriptor will always be set.
    let network = env.get_network_descriptor();
    fetch_root_key_if_needed(env).await?;
    let wallet =
        Identity::get_or_create_wallet_canister(env, network, &identity_name, false).await?;
    Ok(wallet)
}
