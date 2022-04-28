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
use ic_utils::call::SyncCall;
use ic_utils::interfaces::WalletCanister;
use tokio::runtime::Runtime;

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
pub struct WalletOpts {
    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Addresses(list_addresses::AddressesOpts),
    AddController(add_controller::AddControllerOpts),
    Authorize(authorize::AuthorizeOpts),
    Balance(balance::WalletBalanceOpts),
    Controllers(controllers::ControllersOpts),
    Custodians(custodians::CustodiansOpts),
    Deauthorize(deauthorize::DeauthorizeOpts),
    Name(name::NameOpts),
    RemoveController(remove_controller::RemoveControllerOpts),
    Send(send::SendOpts),
    SetName(set_name::SetNameOpts),
    Upgrade(upgrade::UpgradeOpts),
}

pub fn exec(env: &dyn Environment, opts: WalletOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())
        .context("Failed to create AgentEnvironment.")?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Addresses(v) => list_addresses::exec(&agent_env, v).await,
            SubCommand::AddController(v) => add_controller::exec(&agent_env, v).await,
            SubCommand::Authorize(v) => authorize::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::Controllers(v) => controllers::exec(&agent_env, v).await,
            SubCommand::Custodians(v) => custodians::exec(&agent_env, v).await,
            SubCommand::Deauthorize(v) => deauthorize::exec(&agent_env, v).await,
            SubCommand::Name(v) => name::exec(&agent_env, v).await,
            SubCommand::RemoveController(v) => remove_controller::exec(&agent_env, v).await,
            SubCommand::Send(v) => send::exec(&agent_env, v).await,
            SubCommand::SetName(v) => set_name::exec(&agent_env, v).await,
            SubCommand::Upgrade(v) => upgrade::exec(&agent_env, v).await,
        }
    })
}

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
    let network = env.get_network_descriptor().unwrap();
    let wallet = Identity::get_or_create_wallet_canister(env, network, &identity_name, false)
        .await
        .context("Failed to get/create wallet.")?;

    let out: O = wallet
        .query_(method)
        .with_arg(arg)
        .build()
        .call()
        .await
        .context("Query to wallet failed.")?;
    Ok(out)
}

async fn wallet_update<A, O>(env: &dyn Environment, method: &str, arg: A) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let wallet = get_wallet(env)
        .await
        .context("Failed to fetch wallet caller.")?;
    let out: O = wallet
        .update_(method)
        .with_arg(arg)
        .build()
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("Update call to wallet failed.")?;
    Ok(out)
}

async fn get_wallet(env: &dyn Environment) -> DfxResult<WalletCanister<'_>> {
    let identity_name = env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    // Network descriptor will always be set.
    let network = env.get_network_descriptor().unwrap();
    fetch_root_key_if_needed(env)
        .await
        .context("Failed to fetch root key.")?;
    let wallet = Identity::get_or_create_wallet_canister(env, network, &identity_name, false)
        .await
        .context("Failed to fetch wallet.")?;
    Ok(wallet)
}
