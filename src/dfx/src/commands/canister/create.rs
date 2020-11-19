use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::operations::canister::create_canister;
use crate::util::expiry_duration;
use clap::Clap;
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

pub fn exec(env: &dyn Environment, opts: CanisterCreateOpts) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let timeout = expiry_duration();

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    if let Some(canister_name) = opts.canister_name.clone() {
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
        Err(DfxError::CanisterNameMissing())
    }
}
