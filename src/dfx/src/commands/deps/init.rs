use crate::lib::deps::{
    create_init_json_if_not_existed, get_pull_canisters_in_config, get_service_candid_path,
    load_init_json, load_pulled_json, save_init_json, validate_pulled,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::check_candid_file;

use anyhow::{anyhow, bail, Context};
use candid::parser::types::IDLTypes;
use candid::parser::value::IDLValue;
use candid::types::Type;
use candid::{IDLArgs, Principal, TypeEnv};
use clap::Parser;
use fn_error_context::context;
use slog::{info, warn};

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
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There are no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    let pulled_json = load_pulled_json(&project_root)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    create_init_json_if_not_existed(&project_root)?;
    let mut init_json = load_init_json(&project_root)?;

    match opts.canister {
        Some(canister) => {
            let canister_id = match pull_canisters_in_config.get(&canister) {
                Some(canister_id) => *canister_id,
                None => Principal::from_text(&canister).with_context(|| {
                    "The canister is neither a valid Principal nor a name specified in dfx.json"
                })?,
            };

            let idl_path = get_service_candid_path(canister_id)?;
            let (env, _) = check_candid_file(&idl_path)?;
            let candid_args = pulled_json.get_candid_args(&canister_id)?;
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
                        let bytes = args_to_bytes(arg_str, &env, &types)?;
                        init_json.set_init_arg(canister_id, Some(arg_str.to_string()), &bytes);
                    }
                }
                (Some(_), true) => {
                    bail!("Canister {canister_id} takes no init argument. Please rerun without `--argument`");
                }
                (None, false) => {
                    let mut message = format!("Canister {canister_id} requires an init argument. The following info might be helpful:");
                    if let Some(dfx_init) = pulled_json.get_dfx_init(&canister_id)? {
                        message.push_str(&format!("dfx:init    => {dfx_init}"));
                    }
                    let candid_args = pulled_json.get_candid_args(&canister_id)?;
                    message.push_str(&format!("candid:args => {candid_args}"));

                    bail!(message);
                }
                (None, true) => {
                    init_json.set_empty_init(canister_id);
                }
            }
        }
        None => {
            let mut canisters_require_init = vec![];
            for canister_id in pulled_json.canisters.keys() {
                if init_json.contains(canister_id) {
                    info!(logger, "{canister_id} already set init argument.");
                } else {
                    let candid_args = pulled_json.get_candid_args(canister_id)?;
                    let candid_args_idl_types: IDLTypes = candid_args.parse()?;
                    if candid_args_idl_types.args.is_empty() {
                        init_json.set_empty_init(*canister_id);
                    } else {
                        canisters_require_init.push(*canister_id);
                    }
                }
            }
            if !canisters_require_init.is_empty() {
                let mut message = "The following canister(s) require an init argument. Please run `dfx deps init <PRINCIPAL>` to set them individually:".to_string();
                for canister_id in canisters_require_init {
                    message.push_str(&format!("\n{canister_id}"));
                }
                warn!(logger, "{message}");
            }
        }
    }

    save_init_json(&project_root, &init_json)?;
    Ok(())
}

#[context("Failed to validate argument against type defined in candid:args")]
fn args_to_bytes(
    arg_str: &str,
    env: &TypeEnv,
    types: &[Type], // types has been checked to not be empty
) -> DfxResult<Vec<u8>> {
    let first_char = arg_str.chars().next();
    let is_candid_format = first_char.map_or(false, |c| c == '(');
    // If parsing fails and method expects a single value, try parsing as IDLValue.
    // If it still fails, and method expects a text type, send arguments as text.
    let args = arg_str.parse::<IDLArgs>().or_else(|_| {
        if types.len() == 1 && !is_candid_format {
            let is_quote = first_char.map_or(false, |c| c == '"');
            if candid::types::Type::Text == types[0] && !is_quote {
                Ok(IDLValue::Text(arg_str.to_string()))
            } else {
                candid::pretty_parse::<IDLValue>("Candid argument", arg_str)
            }
            .map(|v| IDLArgs::new(&[v]))
        } else {
            candid::pretty_parse::<IDLArgs>("Candid argument", arg_str)
        }
    })?;
    let bytes = args.to_bytes_with_types(env, types)?;
    Ok(bytes)
}
