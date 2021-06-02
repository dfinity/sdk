use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::call_sender;
use crate::lib::operations::canister::deploy_canisters;
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::expiry_duration;

use clap::Clap;
use tokio::runtime::Runtime;

/// Deploys all or a specific canister from the code in your project. By default, all canisters are deployed.
#[derive(Clap)]
pub struct DeployOpts {
    /// Specifies the name of the canister you want to deploy.
    /// If you don’t specify a canister name, all canisters defined in the dfx.json file are deployed.
    canister_name: Option<String>,

    /// Specifies the argument to pass to the method.
    #[clap(long)]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,

    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[clap(long)]
    network: Option<String>,

    /// Specifies the initial cycle balance to deposit into the newly created canister.
    /// The specified amount needs to take the canister create fee into account.
    /// This amount is deducted from the wallet's cycle balance.
    #[clap(long, validator(cycle_amount_validator))]
    with_cycles: Option<String>,

    /// Specify a wallet canister id to perform the call.
    /// If none specified, defaults to use the selected Identity's wallet canister.
    #[clap(long)]
    wallet: Option<String>,

    /// Performs the call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[clap(long, conflicts_with("wallet"))]
    no_wallet: bool,
}

pub fn exec(env: &dyn Environment, opts: DeployOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network)?;

    let timeout = expiry_duration();
    let canister_name = opts.canister_name.as_deref();
    let argument = opts.argument.as_deref();
    let argument_type = opts.argument_type.as_deref();
    let with_cycles = opts.with_cycles.as_deref();

    let runtime = Runtime::new().expect("Unable to create a runtime");

    let default_wallet_proxy = true;
    let call_sender = runtime.block_on(call_sender(
        &env,
        &opts.wallet,
        opts.no_wallet,
        default_wallet_proxy,
    ))?;
    runtime.block_on(fetch_root_key_if_needed(&env))?;

    runtime.block_on(deploy_canisters(
        &env,
        canister_name,
        argument,
        argument_type,
        timeout,
        with_cycles,
        &call_sender,
    ))
}
