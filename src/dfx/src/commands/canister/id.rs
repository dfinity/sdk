use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;

use candid::Principal;
use clap::Parser;

/// Prints the identifier of a canister.
#[derive(Parser)]
pub struct CanisterIdOpts {
    /// Specifies the name of the canister.
    canister: String,
}

pub async fn exec(env: &dyn Environment, opts: CanisterIdOpts) -> DfxResult {
    env.get_config_or_anyhow()?;
    let canister_name = opts.canister.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister_name).or_else(|_| canister_id_store.get(canister_name))?;
    println!("{}", Principal::to_text(&canister_id));
    Ok(())
}
