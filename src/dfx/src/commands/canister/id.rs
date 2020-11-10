use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;

use anyhow::anyhow;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use ic_types::principal::Principal as CanisterId;

/// Prints the identifier of a canister.
#[derive(Clap)]
#[clap(name("id"))]
pub struct CanisterIdOpts {
    /// Specifies the name of the canister to stop.
    /// You must specify either a canister name or the --all option.
    canister_name: String,
}

pub fn construct() -> App<'static> {
    CanisterIdOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts = CanisterIdOpts::from_arg_matches(args);
    env.get_config()
        .ok_or(anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;
    let canister_name = opts.canister_name.as_str();
    let canister_id = CanisterIdStore::for_env(env)?.get(canister_name)?;

    println!("{}", CanisterId::to_text(&canister_id));
    Ok(())
}
