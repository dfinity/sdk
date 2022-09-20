use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::{install_canister, install_canister_wasm};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_init_type};

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use slog::info;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Installs compiled code in a canister.
#[derive(Parser, Clone)]
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
    /// If auto is selected, either install or upgrade will be used depending on if the canister has already been installed.
    #[clap(long, short('m'), default_value("install"),
        possible_values(&["install", "reinstall", "upgrade", "auto"]))]
    mode: String,

    /// Upgrade the canister even if the .wasm did not change.
    #[clap(long)]
    upgrade_unchanged: bool,

    /// Specifies the argument to pass to the method.
    #[clap(long)]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,

    /// Specifies a particular WASM file to install, bypassing the dfx.json project settings.
    #[clap(long, conflicts_with("all"))]
    wasm: Option<PathBuf>,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterInstallOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    let mode = if opts.mode == "auto" {
        None
    } else {
        Some(InstallMode::from_str(&opts.mode).map_err(|err| anyhow!(err))?)
    };
    let mut canister_id_store = CanisterIdStore::for_env(env)?;
    let network = env.get_network_descriptor();

    if mode == Some(InstallMode::Reinstall) && (opts.canister.is_none() || opts.all) {
        bail!("The --mode=reinstall is only valid when specifying a single canister, because reinstallation destroys all data in the canister.");
    }

    if let Some(canister) = opts.canister.as_deref() {
        let config = env.get_config();
        let is_remote = config
            .as_ref()
            .map_or(Ok(false), |config| {
                config
                    .get_config()
                    .is_remote_canister(canister, &network.name)
            })
            .unwrap_or(false);
        if is_remote {
            bail!("Canister '{}' is a remote canister on network '{}', and cannot be installed from here.", canister, &network.name)
        }

        let canister_id =
            Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
        let arguments = opts.argument.as_deref();
        let arg_type = opts.argument_type.as_deref();
        let canister_info = config.as_ref()
            .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))
            .and_then(|config| CanisterInfo::load(config, canister, Some(canister_id)));
        if let Some(wasm_path) = opts.wasm {
            // streamlined version, we can ignore most of the environment
            let mode = mode.context("The install mode cannot be auto when using --wasm")?;
            let install_args = blob_from_arguments(arguments, None, arg_type, &None)?;
            install_canister_wasm(
                env,
                agent,
                canister_id,
                canister_info.as_ref().map(|info| info.get_name()).ok(),
                &install_args,
                mode,
                timeout,
                call_sender,
                fs::read(&wasm_path)
                    .with_context(|| format!("Unable to read {}", wasm_path.display()))?,
            )
            .await
        } else {
            let canister_info = canister_info
                .with_context(|| format!("Failed to load canister info for {}.", canister))?;
            let idl_path = canister_info.get_build_idl_path();
            let init_type = get_candid_init_type(&idl_path);
            let install_args = || blob_from_arguments(arguments, None, arg_type, &init_type);
            install_canister(
                env,
                agent,
                &mut canister_id_store,
                &canister_info,
                &install_args,
                mode,
                timeout,
                call_sender,
                opts.upgrade_unchanged,
                None,
            )
            .await
        }
    } else if opts.all {
        // Install all canisters.
        let config = env.get_config_or_anyhow()?;
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                let canister_is_remote = config
                    .get_config()
                    .is_remote_canister(canister, &network.name)?;
                if canister_is_remote {
                    info!(
                        env.get_logger(),
                        "Skipping canister '{}' because it is remote for network '{}'",
                        canister,
                        &network.name,
                    );

                    continue;
                }
                let canister_id =
                    Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
                let canister_info = CanisterInfo::load(&config, canister, Some(canister_id))?;

                let install_args = || Ok(vec![]);

                install_canister(
                    env,
                    agent,
                    &mut canister_id_store,
                    &canister_info,
                    &install_args,
                    mode,
                    timeout,
                    call_sender,
                    opts.upgrade_unchanged,
                    None,
                )
                .await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
