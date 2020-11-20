use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::create_canister;
use crate::util::expiry_duration;

use anyhow::bail;
use clap::Clap;

/// Creates an empty canister on the Internet Computer and
/// associates the Internet Computer assigned Canister ID to the canister name.
#[derive(Clap)]
#[clap(name("create"))]
pub struct CanisterCreateOpts {
    /// Specifies the canister name. Either this or the --all flag are required.
    canister_name: Option<String>,

    /// Creates all canisters configured in dfx.json.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,
}

pub async fn exec(env: &dyn Environment, opts: CanisterCreateOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    if let Some(canister_name) = opts.canister_name.clone() {
        create_canister(env, canister_name.as_str(), timeout).await
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                create_canister(env, canister_name, timeout).await?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
