use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::expiry_duration;

use anyhow::bail;
use clap::Clap;

/// Creates an empty canister on the Internet Computer and
/// associates the Internet Computer assigned Canister ID to the canister name.
#[derive(Clap)]
pub struct CanisterCreateOpts {
    /// Specifies the canister name. Either this or the --all flag are required.
    canister_name: Option<String>,

    /// Creates all canisters configured in dfx.json.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,

    /// Specifies the initial cycle balance to deposit into the newly created canister.
    /// The specified amount needs to take the canister create fee into account.
    /// This amount is deducted from the wallet's cycle balance.
    #[clap(long, validator(cycle_amount_validator))]
    with_cycles: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: CanisterCreateOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;
    let with_cycles = opts.with_cycles.as_deref();
    if let Some(canister_name) = opts.canister_name.clone() {
        create_canister(env, canister_name.as_str(), timeout, with_cycles).await
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                create_canister(env, canister_name, timeout, with_cycles).await?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
