use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;

use clap::Clap;
use ic_types::principal::Principal as CanisterId;

/// Prints the identifier of a canister.
#[derive(Clap)]
pub struct CanisterIdOpts {
    /// Specifies the name of the canister to stop.
    /// You must specify either a canister name or the --all option.
    canister_name: String,
}

pub async fn exec(env: &dyn Environment, opts: CanisterIdOpts) -> DfxResult {
    env.get_config_or_anyhow()?;
    let canister_name = opts.canister_name.as_str();
    let canister_id = CanisterIdStore::for_env(env)?.get(canister_name)?;
    println!("{}", CanisterId::to_text(&canister_id));
    Ok(())
}
