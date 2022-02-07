use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::create_agent_environment;

use clap::Clap;

/// Generate stubs for remote canisters from their .did declarations
#[derive(Clap)]
pub struct GenerateMockOpts {
    /// Specifies the name of the canister to build.
    /// If you do not specify a canister name, generates mocks for all canisters that have no implementation yet.
    canister_name: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: GenerateMockOpts) -> DfxResult {
    let env = create_agent_environment(env, None)?;

    todo!();

    Ok(())
}
