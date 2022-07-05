use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::provider::create_agent_environment;
use crate::util::check_candid_file;

use anyhow::Context;
use clap::Parser;
use slog::info;

/// Generate bindings for remote canisters from their .did declarations
#[derive(Parser)]
pub struct GenerateBindingOpts {
    /// Specifies the name of the canister to generate bindings for.
    /// You must specify either canister name/id or the --all option.
    /// Generates bindings into <canister-name>.main from <canister-name>.remote.candid
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

    //collects specified canister, or all if canister is None (= --all is set)
    let canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(opts.canister.as_deref())?;
    let canister_pool = CanisterPool::load(&env, false, &canister_names)?;

    for canister in canister_pool.get_canister_list() {
        let info = canister.get_info();
        if let Some(candid) = info.get_remote_candid() {
            let main_optional = info.get_main_file();
            if let Some(main) = main_optional {
                if !candid.exists() {
                    info!(
                        log,
                        "Candid file {} for canister {} does not exist. Skipping.",
                        candid.to_string_lossy(),
                        canister.get_name()
                    );
                    continue;
                }
                if main.exists() {
                    if opts.overwrite {
                        info!(
                            log,
                            "Overwriting main file {} of canister {}.",
                            main.display(),
                            canister.get_name()
                        );
                    } else {
                        info!(
                            log,
                            "Main file {} of canister {} already exists. Skipping. Use --overwrite if you want to re-create it.",
                            main.display(),
                            canister.get_name()
                        );
                        continue;
                    }
                }
                let (type_env, did_types) = check_candid_file(&candid)?;
                let extension = main.extension().unwrap_or_default();
                let bindings = if extension == "mo" {
                    Some(candid::bindings::motoko::compile(&type_env, &did_types))
                } else if extension == "rs" {
                    Some(candid::bindings::rust::compile(&type_env, &did_types))
                } else if extension == "js" {
                    Some(candid::bindings::javascript::compile(&type_env, &did_types))
                } else if extension == "ts" {
                    Some(candid::bindings::typescript::compile(&type_env, &did_types))
                } else {
                    info!(
                        log,
                        "Unsupported filetype found in {}.main: {}. Use one of the following: .mo, .rs, .js, .ts",
                        canister.get_name(),
                        main.display()
                    );
                    None
                };

                if let Some(bindings_string) = bindings {
                    std::fs::write(&main, &bindings_string).with_context(|| {
                        format!("Failed to write bindings to {}.", main.display())
                    })?;
                    info!(
                        log,
                        "Generated {} using {} for canister {}.",
                        main.display(),
                        candid.display(),
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
