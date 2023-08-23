use crate::lib::deps::{
    create_init_json_if_not_existed, get_canister_prompt, get_pull_canister_or_principal,
    get_pull_canisters_in_config, get_pulled_service_candid_path, load_init_json, load_pulled_json,
    save_init_json, validate_pulled, InitJson, PulledJson,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::{check_candid_file, fuzzy_parse_argument};
use anyhow::{anyhow, bail};
use candid::parser::types::IDLTypes;
use candid::Principal;
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
            set_init(&canister_id, &mut init_json, &pulled_json, &opts)?;
        }
        None => try_set_empty_init_for_all(logger, &mut init_json, &pulled_json)?,
    }

    save_init_json(&project_root, &init_json)?;
    Ok(())
}

fn set_init(
    canister_id: &Principal,
    init_json: &mut InitJson,
    pulled_json: &PulledJson,
    opts: &DepsInitOpts,
) -> DfxResult {
    let pulled_canister = pulled_json
        .canisters
        .get(canister_id)
        .ok_or_else(|| anyhow!("Failed to find {canister_id} entry in pulled.json"))?;
    let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
    let idl_path = get_pulled_service_candid_path(canister_id)?;
    let (env, _) = check_candid_file(&idl_path)?;
    let candid_args = pulled_json.get_candid_args(canister_id)?;
    let candid_args_idl_types: IDLTypes = candid_args.parse()?;
    let mut types = vec![];
    for ty in candid_args_idl_types.args.iter() {
        types.push(env.ast_to_type(ty)?);
    }

    let arguments = opts.argument.as_deref();
    let arg_type = opts.argument_type.as_deref();

    match (arguments, types.is_empty()) {
        (Some(arg_str), false) => {
            if arg_type == Some("raw") {
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
            let mut message = format!("Canister {canister_prompt} requires an init argument. The following info might be helpful:");
            let init = pulled_json.get_init(canister_id)?;
            message.push_str(&format!("\ninit => {init}"));

            let candid_args = pulled_json.get_candid_args(canister_id)?;
            message.push_str(&format!("\ncandid:args => {candid_args}"));

            bail!(message);
        }
        (None, true) => {
            init_json.set_empty_init(canister_id);
        }
    }
    Ok(())
}

fn try_set_empty_init_for_all(
    logger: &Logger,
    init_json: &mut InitJson,
    pulled_json: &PulledJson,
) -> DfxResult {
    let mut canisters_require_init = vec![];
    for (canister_id, pulled_canister) in &pulled_json.canisters {
        let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
        if init_json.contains(canister_id) {
            info!(logger, "{canister_prompt} already set init argument.");
        } else {
            let candid_args = pulled_json.get_candid_args(canister_id)?;
            let candid_args_idl_types: IDLTypes = candid_args.parse()?;
            if candid_args_idl_types.args.is_empty() {
                init_json.set_empty_init(canister_id);
            } else {
                canisters_require_init.push(canister_prompt);
            }
        }
    }
    if !canisters_require_init.is_empty() {
        let mut message = "The following canister(s) require an init argument. Please run `dfx deps init <NAME/PRINCIPAL>` to set them individually:".to_string();
        for canister_prompt in canisters_require_init {
            message.push_str(&format!("\n{canister_prompt}"));
        }
        warn!(logger, "{message}");
    }
    Ok(())
}
