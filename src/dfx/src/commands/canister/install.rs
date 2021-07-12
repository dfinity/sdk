use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::install_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_init_type};

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::{Agent, AgentError};
use ic_types::Principal;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use std::str::FromStr;

/// Deploys compiled code as a canister on the Internet Computer.
#[derive(Clap, Clone)]
pub struct CanisterInstallOpts {
    /// Specifies the canister to deploy. You must specify either canister name/id or the --all option.
    canister: Option<String>,

    /// Deploys all canisters configured in the project dfx.json files.
    #[clap(long, required_unless_present("canister"))]
    all: bool,

    /// Specifies not to wait for the result of the call to be returned by polling the replica. Instead return a response ID.
    #[clap(long)]
    async_call: bool,

    /// Specifies the type of deployment. You can set the canister deployment modes to install, reinstall, or upgrade.
    #[clap(long, short('m'), default_value("install"),
        possible_values(&["install", "reinstall", "upgrade"]))]
    mode: String,

    /// Specifies the argument to pass to the method.
    #[clap(long)]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterInstallOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    let mode = InstallMode::from_str(opts.mode.as_str()).map_err(|err| anyhow!(err))?;
    let canister_id_store = CanisterIdStore::for_env(env)?;

    if let Some(canister) = opts.canister.as_deref() {
        let canister_id =
            Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
        let canister_info = CanisterInfo::load(&config, canister, Some(canister_id))?;

        let maybe_path = canister_info.get_output_idl_path();
        let init_type = maybe_path.and_then(|path| get_candid_init_type(&path));
        let arguments = opts.argument.as_deref();
        let arg_type = opts.argument_type.as_deref();
        let install_args = blob_from_arguments(arguments, None, arg_type, &init_type)?;
        let installed_module_hash =
            read_module_hash(agent, &canister_id_store, &canister_info).await?;

        install_canister(
            env,
            &agent,
            &canister_info,
            &install_args,
            mode,
            timeout,
            call_sender,
            installed_module_hash,
        )
        .await
    } else if opts.all {
        // Install all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                let canister_id =
                    Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
                let canister_info = CanisterInfo::load(&config, canister, Some(canister_id))?;
                let installed_module_hash =
                    read_module_hash(agent, &canister_id_store, &canister_info).await?;

                let install_args = [];

                install_canister(
                    env,
                    &agent,
                    &canister_info,
                    &install_args,
                    mode,
                    timeout,
                    call_sender,
                    installed_module_hash,
                )
                .await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}

async fn read_module_hash(
    agent: &Agent,
    canister_id_store: &CanisterIdStore,
    canister_info: &CanisterInfo,
) -> DfxResult<Option<Vec<u8>>> {
    match canister_id_store.find(canister_info.get_name()) {
        Some(canister_id) => {
            match agent
                .read_state_canister_info(canister_id, "module_hash")
                .await
            {
                Ok(installed_module_hash) => Ok(Some(installed_module_hash)),
                // If the canister is empty, this path does not exist.
                // The replica doesn't support negative lookups, therefore if the canister
                // is empty, the replica will return lookup_path([], Pruned _) = Unknown
                Err(AgentError::LookupPathUnknown(_)) | Err(AgentError::LookupPathAbsent(_)) => {
                    Ok(None)
                }
                Err(x) => bail!(x),
            }
        }
        None => Ok(None),
    }
}
