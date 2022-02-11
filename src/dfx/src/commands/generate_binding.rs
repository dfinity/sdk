use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::provider::create_agent_environment;
use crate::util::check_candid_file;

use clap::Clap;
use slog::info;
use std::path::Path;

/// Generate bindings for remote canisters from their .did declarations
#[derive(Clap)]
pub struct GenerateBindingOpts {
    /// Specifies the name of the canister to generate bindings for.
    /// You must specify either canister name/id or the --all option.
    canister: Option<String>,

    /// Builds bindings for all canisters.
    #[clap(long, required_unless_present("canister"))]
    // destructive operations (see --overwrite) can happen
    // therefore it is safer to require the explicit --all flag
    #[allow(dead_code)]
    all: bool,

    /// Overwrite main file if it already exists.
    #[clap(long)]
    overwrite: bool,
}

pub fn exec(env: &dyn Environment, opts: GenerateBindingOpts) -> DfxResult {
    let env = create_agent_environment(env, None)?;
    let config = env.get_config_or_anyhow()?;
    let log = env.get_logger();

    //fetches specified canister, or all if canister is None (= --all is set)
    let canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(opts.canister.as_deref())?;
    let canister_pool = CanisterPool::load(&env, false, &canister_names)?;

    for canister in canister_pool.get_canister_list() {
        let info = canister.get_info();
        if let Some(candid) = info.get_remote_candid() {
            let main_optional: Option<String> = info.get_extra_optional("main")?;
            if let Some(main) = main_optional {
                let main_path = Path::new(&main);
                let candid_path = Path::new(&candid);
                if !candid_path.exists() {
                    info!(
                        log,
                        "Candid file {} for canister {} does not exist. Skipping.",
                        candid,
                        canister.get_name()
                    );
                    continue;
                }
                if main_path.exists() {
                    if opts.overwrite {
                        info!(
                            log,
                            "Overwriting main file {} of canister {}.",
                            main,
                            canister.get_name()
                        );
                    } else {
                        info!(
                            log,
                            "Main file {} of canister {} already exists. Skipping. Use --overwrite if you want to re-create it.",
                            main,
                            canister.get_name()
                        );
                        continue;
                    }
                }
                let (type_env, did_types) = check_candid_file(&candid_path)?;
                let bindings = if main.ends_with(&".mo") {
                    Some(candid::bindings::motoko::compile(&type_env, &did_types))
                } else if main.ends_with(&".rs") {
                    Some(candid::bindings::rust::compile(&type_env, &did_types))
                } else if main.ends_with(&".js") {
                    Some(candid::bindings::javascript::compile(&type_env, &did_types))
                } else if main.ends_with(&".ts") {
                    Some(candid::bindings::typescript::compile(&type_env, &did_types))
                } else {
                    info!(
                        log,
                        "Unsupported filetype found in main: {}. Use one of the following: .mo, .rs, .js, .ts",
                        main
                    );
                    None
                };

                if let Some(bindings_string) = bindings {
                    std::fs::write(&main_path, &bindings_string)?;
                    info!(
                        log,
                        "Generated {} using {} for canister {}.",
                        main,
                        candid,
                        canister.get_name()
                    )
                }
            } else {
                info!(
                    log,
                    "Canister {} is missing attribute 'main'. Without this attribute I do not know where to generate the bindings.",
                    canister.get_name()
                );
            }
        } else {
            info!(
                log,
                "Canister {} is not remote. Skipping.",
                canister.get_name()
            );
        }
    }

    Ok(())
}
