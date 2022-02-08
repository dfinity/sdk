use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::{Canister, CanisterPool};
use crate::lib::provider::create_agent_environment;

use clap::Clap;

/// Generate .mo stubs for remote canisters from their .did declarations
#[derive(Clap)]
pub struct GenerateMockOpts {
    /// Specifies the name of the canister to generate stubs for.
    /// If you do not specify a canister name, generates mocks for all canisters that don't have a main file.
    canister_name: Option<String>,

    /// Overwrite main .mo file if it already exists.
    #[clap(long)]
    overwrite: bool,
}

pub fn exec(env: &dyn Environment, opts: GenerateMockOpts) -> DfxResult {
    let env = create_agent_environment(env, None)?;

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    //Option can be None which means generate stubs for all canisters.
    let canister_names = config.get_config().get_canister_names_with_dependencies(opts.canister_name.as_deref())?;
    let canister_pool = CanisterPool::load(&env, false, &canister_names)?;

    for canister in canister_pool.get_canister_list() {
        println!("Canisters: {:?}", canister.canister_id());
    }
    // filter such that only canisters without main.mo are left
    todo!();

    Ok(())
}

fn qualifies_for_generating(canister: &Canister) -> bool {
    false
    //todo!
}