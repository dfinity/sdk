use crate::lib::error::DfxResult;
use crate::util::blob_from_arguments;
use crate::{lib::environment::Environment, util::get_candid_init_type};

use anyhow::Context;
use candid::Principal;
use clap::Parser;
use slog::{info, warn};

use super::{
    get_pull_canisters_in_config, get_pulled_candid_path, read_init_json, read_pulled_json,
    validate_pulled, write_init_json,
};

/// Set init argument for a pulled canister.
#[derive(Parser)]
pub struct DepsInitOpts {
    /// Name of the pulled canister (as defined in dfx.json) or its Principal.
    /// If not specified, all pulled canisters will be set.
    canister: Option<String>,

    /// Specifies the init argument.
    #[clap(long, requires("canister"))]
    argument: Option<String>,

    /// Specifies the data type of the init argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: DepsInitOpts) -> DfxResult {
    let logger = env.get_logger();

    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    let pulled_json = read_pulled_json(env)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    let mut init_json = read_init_json(env)?;

    match opts.canister {
        Some(canister) => {
            let canister_id = match pull_canisters_in_config.get(&canister) {
                Some(canister_id) => *canister_id,
                None => Principal::from_text(&canister).with_context(|| {
                    "The canister is neither a valid Principal nor a name specified in dfx.json"
                })?,
            };

            let idl_path = get_pulled_candid_path(canister_id)?;
            let init_type = get_candid_init_type(&idl_path);
            let arguments = opts.argument.as_deref();
            let arg_type = opts.argument_type.as_deref();
            // TODO: improve the error message here
            let arg_raw = blob_from_arguments(arguments, None, arg_type, &init_type)?;
            if arg_type == Some("raw") {
                init_json.set_init_arg(canister_id, None, &arg_raw);
            } else {
                init_json.set_init_arg(canister_id, opts.argument, &arg_raw);
            }
        }
        None => {
            let mut canisters_require_init = vec![];
            for canister_id in pulled_json.canisters.keys() {
                if init_json.contains(canister_id) {
                    info!(logger, "{canister_id} already set init argument.");
                } else {
                    let idl_path = get_pulled_candid_path(*canister_id)?;
                    let init_type = get_candid_init_type(&idl_path);
                    match blob_from_arguments(None, None, None, &init_type) {
                        Ok(bytes) => {
                            assert!(bytes.is_empty());
                            init_json.set_empty_init(*canister_id);
                        }
                        Err(_) => {
                            canisters_require_init.push(*canister_id);
                        }
                    }
                }
            }
            warn!(logger, "Following canister(s) require init argument, please run `dfx deps init <PRINCIPAL>` to set them individually:");
            for canister_id in canisters_require_init {
                println!("{canister_id}");
            }
        }
    }

    write_init_json(env, &init_json)?;
    Ok(())
}
