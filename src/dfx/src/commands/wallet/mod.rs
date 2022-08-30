use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;
use crate::NetworkOpt;

use anyhow::Context;
use candid::utils::ArgumentDecoder;
use candid::CandidType;
use clap::Parser;
use fn_error_context::context;
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
mod redeem_faucet_coupon;
mod remove_controller;
mod send;
mod set_name;
mod upgrade;

/// Helper commands to manage the user's cycles wallet.
#[derive(Parser)]
#[clap(name("wallet"))]
pub struct WalletOpts {
    #[clap(flatten)]
    network: NetworkOpt,

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
    RedeemFaucetCoupon(redeem_faucet_coupon::RedeemFaucetCouponOpts),
    RemoveController(remove_controller::RemoveControllerOpts),
    Send(send::SendOpts),
    SetName(set_name::SetNameOpts),
    Upgrade(upgrade::UpgradeOpts),
}

pub fn exec(env: &dyn Environment, opts: WalletOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.network)?;
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
            SubCommand::RedeemFaucetCoupon(v) => redeem_faucet_coupon::exec(&agent_env, v).await,
            SubCommand::RemoveController(v) => remove_controller::exec(&agent_env, v).await,
            SubCommand::Send(v) => send::exec(&agent_env, v).await,
            SubCommand::SetName(v) => set_name::exec(&agent_env, v).await,
            SubCommand::Upgrade(v) => upgrade::exec(&agent_env, v).await,
        }
    })
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
    let wallet = Identity::get_or_create_wallet_canister(env, network, &identity_name).await?;

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
    let wallet = Identity::get_or_create_wallet_canister(env, network, &identity_name).await?;
    Ok(wallet)
}
