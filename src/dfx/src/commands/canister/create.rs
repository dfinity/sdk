use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::operations::canister::create_canister;
use crate::util::expiry_duration;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

/// Creates an empty canister on the Internet Computer and
/// associates the Internet Computer assigned Canister ID to the canister name.
#[derive(Clap)]
pub struct CanisterCreateOpts {
    /// Specifies the canister name. Either this or the --all flag are required.
    #[clap(long, required_unless_present("all"))]
    canister_name: String,

    /// Creates all canisters configured in dfx.json.
    #[clap(long, required_unless_present("canister_name"))]
    all: bool,
}

pub fn construct() -> App<'static> {
    CanisterCreateOpts::into_app().name("create")
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CanisterCreateOpts = CanisterCreateOpts::from_arg_matches(args);
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let timeout = expiry_duration();

    if let Some(canister_name) = Some(opts.canister_name.as_str()) {
        create_canister(env, canister_name, timeout)?;
        Ok(())
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                create_canister(env, canister_name, timeout)?;
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}
