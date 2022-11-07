use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{GetSubnetTypesToSubnetsResult, MAINNET_CYCLE_MINTER_CANISTER_ID};
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::{anyhow, Context};
use candid::{Decode, Encode, Principal};
use clap::Parser;

const GET_SUBNET_TYPES_TO_SUBNETS_METHOD: &str = "get_subnet_types_to_subnets";

/// Show available subnet types in the cycles minting canister.
#[derive(Parser)]
pub struct ShowSubnetTypesOpts {
    #[clap(long)]
    /// Canister ID of the cycles minting canister.
    cycles_minting_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: ShowSubnetTypesOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let cycles_minting_canister_id = opts
        .cycles_minting_canister_id
        .unwrap_or(MAINNET_CYCLE_MINTER_CANISTER_ID);

    let result = agent
        .update(
            &cycles_minting_canister_id,
            GET_SUBNET_TYPES_TO_SUBNETS_METHOD,
        )
        .with_arg(Encode!(&()).context("Failed to encode get_subnet_types_to_subnets arguments.")?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("get_subnet_types_to_subnets call failed.")?;
    let result = Decode!(&result, GetSubnetTypesToSubnetsResult)
        .context("Failed to decode get_subnet_types_to_subnets response")?;

    let available_subnet_types: Vec<String> = result.data.into_iter().map(|(x, _)| x).collect();

    println!("{:?}", available_subnet_types);

    Ok(())
}
