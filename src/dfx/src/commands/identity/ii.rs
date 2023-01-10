use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::anyhow;
use candid::CandidType;
use candid::Encode;
use clap::Clap;
use ic_types::principal::Principal;
use regex::Regex;
use tokio::runtime::Runtime;

/// Adds the current identity as a “device” to an Internet Identity
///
/// This command will produce a link that you can open on a device registered with your
/// Internet Identity.
#[derive(Clap)]
pub struct RegisterWithOpts {
    /// Your user number
    user_number: u64,

    /// Internet Identity URL (https://identity.ic0.app/ by default)
    #[clap(long)]
    url: Option<String>,
}

pub fn register(env: &dyn Environment, opts: RegisterWithOpts) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let public_key = identity
        .as_ref()
        .public_key()
        .map_err(|err| anyhow!("{}", err))?;
    let base = opts
        .url
        .unwrap_or_else(|| "https://identity.ic0.app/".to_string());

    println!(
        "{}#device={};{}",
        base,
        opts.user_number,
        hex::encode(public_key)
    );
    Ok(())
}

/// Adds other devices to your Internet Identity
///
/// This only works after the present identity has been registered with your Internet
/// Identity.
///
/// It will accept a device add link, as produced by the Internet Identity, and approve this.
///
/// Do not pass any untrustworthy links here!
#[derive(Clap)]
pub struct AddDeviceToOpts {
    /// The add-device link
    #[clap(long)]
    link: String,

    /// The alias to use
    #[clap(long)]
    alias: String,

    /// The canister id of the Internet Identity.
    /// (rdmx6-jaaaa-aaaaa-aaadq-cai by default)
    #[clap(long)]
    canister_id: Option<String>,
}

#[derive(CandidType)]
pub struct DeviceData {
    pub pubkey: Vec<u8>,
    pub alias: String,
    pub credential_id: Option<Vec<u8>>,
}

pub fn add_device(
    env: &dyn Environment,
    opts: AddDeviceToOpts,
    network: Option<String>,
) -> DfxResult {
    let re = Regex::new(r".*#device=(\d+);([a-fA-F0-9]+)(?:;([a-fA-F0-9]+))?$").unwrap();
    let cap = re
        .captures(&opts.link)
        .ok_or_else(|| anyhow!("Cannot parse the link"))?;

    let alias = opts.alias;

    // these parses will all succeed, due to the regex
    let user_number: u64 = cap.get(1).unwrap().as_str().parse().unwrap();
    let pubkey: Vec<u8> = hex::decode(cap.get(2).unwrap().as_str()).unwrap();
    let credential_id: Option<Vec<u8>> = cap.get(3).map(|m| hex::decode(m.as_str()).unwrap());

    let canister_id = Principal::from_text(
        opts.canister_id
            .unwrap_or_else(|| "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string()),
    )?;

    let agent_env = create_agent_environment(env, network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        let agent = agent_env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

        fetch_root_key_if_needed(&agent_env).await?;

        let device_data = DeviceData {
            pubkey,
            alias,
            credential_id,
        };

        let _result = agent
            .update(&canister_id, "add")
            .with_arg(Encode!(&user_number, &device_data)?)
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await?;

        Ok(())
    })
}
