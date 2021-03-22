use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_exponential_backoff;
use crate::util::clap::validators;
use crate::util::print_idl_blob;

use anyhow::{anyhow, Context};
use clap::Clap;
use delay::Waiter;
use ic_agent::AgentError;
use ic_types::Principal;
use std::convert::TryFrom;
use std::str::FromStr;

/// Get the hash of a canister’s WASM module and its current controller in a certified way.
#[derive(Clap)]
pub struct ReadStateOpts {
    /// Specifies the name or id of the canister to get its certified canister information.
    canister_name: String,
}

pub async fn exec(env: &dyn Environment, opts: ReadStateOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let callee_canister = opts.canister_name.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let canister_id = match Principal::from_text(callee_canister) {
        Ok(id) => {
            if let Some(canister_name) = canister_id_store.get_name(callee_canister) {
                let config = env.get_config_or_anyhow()?;
                let canister_info = CanisterInfo::load(&config, canister_name, Some(id))?;
                canister_info.get_canister_id()?
            } else {
                id
            }
        }
        Err(_) => {
            canister_id_store.get(callee_canister)?
        }
    };


    fetch_root_key_if_needed(env).await?;
    let controller_blob = agent.read_state_canister_info(canister_id.clone(), "controller").await?;
    let controller = Principal::try_from(controller_blob)?.to_text();
    let module_hash_blob = agent.read_state_canister_info(canister_id, "module_hash").await?;
    let encoded_hex = hex::encode(&module_hash_blob);

    println!("Controller: {}\nModule hash: 0x{}", controller, encoded_hex);

    Ok(())
}
