use crate::lib::deps::{
    create_init_json_if_not_existed, get_canister_prompt, get_pull_canister_or_principal,
    get_pull_canisters_in_config, get_pulled_service_candid_path, load_init_json, load_pulled_json,
    save_init_json, validate_pulled, InitJson, PulledJson,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::fuzzy_parse_argument;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use candid_parser::{types::IDLTypes, typing::ast_to_type, utils::CandidSource};
use clap::Parser;
use slog::{info, warn, Logger};

/// Set init arguments for pulled dependencies.
#[derive(Parser)]
pub struct DepsInitOpts {
    /// Name of the pulled canister (as defined in dfx.json) or its Principal.
    /// If not specified, all pulled canisters will be set.
    canister: Option<String>,

    /// Specifies the init argument.
    #[arg(long, requires("canister"))]
    argument: Option<String>,

    /// Specifies the data type of the init argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    argument_type: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: DepsInitOpts) -> DfxResult {
    let logger = env.get_logger();
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There are no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    let pulled_json = load_pulled_json(&project_root)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    create_init_json_if_not_existed(&project_root)?;
    let mut init_json = load_init_json(&project_root)?;

    match &opts.canister {
        Some(canister) => {
            let canister_id =
                get_pull_canister_or_principal(canister, &pull_canisters_in_config, &pulled_json)?;
            set_init(
                logger,
                &canister_id,
                &mut init_json,
                &pulled_json,
                opts.argument.as_deref(),
                opts.argument_type.as_deref(),
            )?;
        }
        None => {
            // try_set_empty_init_for_all(logger, &mut init_json, &pulled_json)?,
            let mut canisters_require_init = vec![];
            for (canister_id, pulled_canister) in &pulled_json.canisters {
                if set_init(
                    logger,
                    canister_id,
                    &mut init_json,
                    &pulled_json,
                    None,
                    None,
                )
                .is_err()
                {
                    let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
                    canisters_require_init.push(canister_prompt);
                }
            }
            if !canisters_require_init.is_empty() {
                let mut message = "The following canister(s) require an init argument. Please run `dfx deps init <NAME/PRINCIPAL>` to set them individually:".to_string();
                for canister_prompt in canisters_require_init {
                    message.push_str(&format!("\n{canister_prompt}"));
                }
                warn!(logger, "{message}");
            }
        }
    }

    save_init_json(&project_root, &init_json)?;
    Ok(())
}

fn set_init(
    logger: &Logger,
    canister_id: &Principal,
    init_json: &mut InitJson,
    pulled_json: &PulledJson,
    argument_from_cli: Option<&str>,
    argument_type_from_cli: Option<&str>,
) -> DfxResult {
    let pulled_canister = pulled_json
        .canisters
        .get(canister_id)
        .ok_or_else(|| anyhow!("Failed to find {canister_id} entry in pulled.json"))?;
    let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
    let idl_path = get_pulled_service_candid_path(canister_id)?;
    let (env, _) = CandidSource::File(&idl_path).load()?;
    let candid_args = pulled_json.get_candid_args(canister_id)?;
    let candid_args_idl_types: IDLTypes = candid_args.parse()?;
    let mut types = vec![];
    for ty in candid_args_idl_types.args.iter() {
        types.push(ast_to_type(&env, ty)?);
    }

    match (argument_from_cli, types.is_empty()) {
        (Some(arg_str), false) => {
            if argument_type_from_cli == Some("raw") {
                let bytes = hex::decode(arg_str)
                    .map_err(|e| anyhow!("Argument is not a valid hex string: {}", e))?;
                init_json.set_init_arg(canister_id, None, &bytes);
            } else {
                let bytes = fuzzy_parse_argument(arg_str, &env, &types)?;
                init_json.set_init_arg(canister_id, Some(arg_str.to_string()), &bytes);
            }
        }
        (Some(_), true) => {
            bail!("Canister {canister_prompt} takes no init argument. Please rerun without `--argument`");
        }
        (None, false) => {
            // No argument provided from CLI but the canister requires an init argument.
            // Try to set the init argument in the following order:
            // 1. If `init.json` already contains the canister, do nothing.
            // 2. If the canister provides an `init_arg`, use it.
            // 3. Try "(null)" which works for canisters with top-level `opt`. This behavior is consistent with `dfx deploy`.
            // 4. Bail.
            let init_guide = pulled_json.get_init_guide(canister_id)?;
            let candid_args = pulled_json.get_candid_args(canister_id)?;
            let help_message = format!("init_guide => {init_guide}\ncandid:args => {candid_args}");

            if init_json.contains(canister_id) {
                info!(
                    logger,
                    "Canister {canister_prompt} already set init argument."
                );
            } else if let Some(init_arg) = pulled_json.get_init_arg(canister_id)? {
                let bytes = fuzzy_parse_argument(init_arg, &env, &types).with_context(|| {
                    format!(
                        "Pulled canister {canister_prompt} provided an invalid `init_arg`.
Please try to set an init argument with `--argument` option.
The following info might be helpful:
{help_message}"
                    )
                })?;
                init_json.set_init_arg(canister_id, Some(init_arg.to_string()), &bytes);
            } else if let Ok(bytes) = fuzzy_parse_argument("(null)", &env, &types) {
                init_json.set_init_arg(canister_id, Some("(null)".to_string()), &bytes);
                info!(
                    logger,
                    "Canister {canister_prompt} set to empty init argument."
                );
            } else {
                bail!("Canister {canister_prompt} requires an init argument. The following info might be helpful:\n{help_message}");
            }
        }
        (None, true) => {
            init_json.set_empty_init(canister_id);
        }
    }
    Ok(())
}

// fn try_set_empty_init_for_all(
//     logger: &Logger,
//     init_json: &mut InitJson,
//     pulled_json: &PulledJson,
// ) -> DfxResult {
//     let mut canisters_require_init = vec![];
//     for (canister_id, pulled_canister) in &pulled_json.canisters {
//         let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
//         if init_json.contains(canister_id) {
//             info!(logger, "{canister_prompt} already set init argument.");
//         } else {
//             let candid_args = pulled_json.get_candid_args(canister_id)?;
//             let candid_args_idl_types: IDLTypes = candid_args.parse()?;
//             if candid_args_idl_types.args.is_empty() {
//                 init_json.set_empty_init(canister_id);
//             } else {
//                 canisters_require_init.push(canister_prompt);
//             }
//         }
//     }
//     if !canisters_require_init.is_empty() {
//         let mut message = "The following canister(s) require an init argument. Please run `dfx deps init <NAME/PRINCIPAL>` to set them individually:".to_string();
//         for canister_prompt in canisters_require_init {
//             message.push_str(&format!("\n{canister_prompt}"));
//         }
//         warn!(logger, "{message}");
//     }
//     Ok(())
// }
