use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::create_canister;
use crate::util::expiry_duration;

use anyhow::bail;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use tokio::runtime::Runtime;

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

pub fn construct() -> App<'static> {
    CanisterCreateOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CanisterCreateOpts = CanisterCreateOpts::from_arg_matches(args);
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    if let Some(canister_name) = opts.canister_name {
        runtime.block_on(create_canister(env, canister_name.as_str(), timeout))?;
        Ok(())
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                runtime.block_on(create_canister(env, canister_name, timeout))?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
