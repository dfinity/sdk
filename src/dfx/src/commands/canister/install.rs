use crate::lib::deps::get_pull_canisters_in_config;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::install_canister::install_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::file_or_stdin_parser;
use crate::{
    lib::canister_info::CanisterInfo,
    util::{arguments_from_file, blob_from_arguments},
};
use dfx_core::canister::install_canister_wasm;
use dfx_core::identity::CallSender;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use slog::info;
use std::path::PathBuf;
use std::str::FromStr;

/// Installs compiled code in a canister.
#[derive(Parser, Clone)]
pub struct CanisterInstallOpts {
    /// Specifies the canister to deploy. You must specify either canister name/id or the --all option.
    canister: Option<String>,

    /// Deploys all canisters configured in the project dfx.json files.
    #[arg(long, required_unless_present("canister"), conflicts_with("argument"))]
    all: bool,

    /// Specifies not to wait for the result of the call to be returned by polling the replica. Instead return a response ID.
    #[arg(long)]
    async_call: bool,

    /// Specifies the type of deployment. You can set the canister deployment modes to install, reinstall, or upgrade.
    /// If auto is selected, either install or upgrade will be used depending on if the canister has already been installed.
    #[arg(long, short, default_value("install"),
        value_parser = ["install", "reinstall", "upgrade", "auto"])]
    mode: String,

    /// Upgrade the canister even if the .wasm did not change.
    #[arg(long)]
    upgrade_unchanged: bool,

    /// Specifies the argument to pass to the method.
    #[arg(long, conflicts_with("argument_file"))]
    argument: Option<String>,

    /// Specifies the file from which to read the argument to pass to the method.
    #[arg(
        long,
        value_parser = file_or_stdin_parser,
        conflicts_with("argument")
    )]
    argument_file: Option<PathBuf>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    argument_type: Option<String>,

    /// Specifies a particular WASM file to install, bypassing the dfx.json project settings.
    #[arg(long, conflicts_with("all"))]
    wasm: Option<PathBuf>,

    /// Output environment variables to a file in dotenv format (without overwriting any user-defined variables, if the file already exists).
    output_env_file: Option<PathBuf>,

    /// Skips yes/no checks by answering 'yes'. Such checks usually result in data loss,
    /// so this is not recommended outside of CI.
    #[arg(long, short)]
    yes: bool,

    /// Skips upgrading the asset canister, to only install the assets themselves.
    #[arg(long)]
    no_asset_upgrade: bool,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterInstallOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let mode = if opts.mode == "auto" {
        None
    } else {
        Some(InstallMode::from_str(&opts.mode).map_err(|err| anyhow!(err))?)
    };
    let mut canister_id_store = env.get_canister_id_store()?;
    let network = env.get_network_descriptor();

    if mode == Some(InstallMode::Reinstall) && (opts.canister.is_none() || opts.all) {
        bail!("The --mode=reinstall is only valid when specifying a single canister, because reinstallation destroys all data in the canister.");
    }

    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;

    let config = env.get_config_or_anyhow()?;
    let config_interface = config.get_config();
    let env_file = config.get_output_env_file(opts.output_env_file)?;

    if let Some(canister) = opts.canister.as_deref() {
        let arguments_from_file = opts
            .argument_file
            .map(|v| arguments_from_file(&v))
            .transpose()?;
        let arguments = opts.argument.as_deref();
        let argument_from_cli = arguments_from_file.as_deref().or(arguments);
        let arg_type = opts.argument_type.as_deref();

        // `opts.canister` is a Principal (canister ID)
        if let Ok(canister_id) = Principal::from_text(canister) {
            if let Some(wasm_path) = &opts.wasm {
                let args = blob_from_arguments(argument_from_cli, None, arg_type, &None)?;
                let wasm_module = dfx_core::fs::read(wasm_path)?;
                let mode = mode.context("The install mode cannot be auto when using --wasm")?;
                install_canister_wasm(
                    env.get_agent(),
                    canister_id,
                    None,
                    &args,
                    mode,
                    call_sender,
                    wasm_module,
                    opts.yes,
                    env.get_logger(),
                )
                .await?;
                Ok(())
            } else {
                bail!("When installing a canister by its ID, you must specify `--wasm` option.")
            }
        } else {
            // `opts.canister` is not a canister ID, but a canister name
            if pull_canisters_in_config.contains_key(canister) {
                bail!(
                    "{0} is a pull dependency. Please deploy it using `dfx deps deploy {0}`",
                    canister
                );
            }
            if config_interface.is_remote_canister(canister, &network.name)? {
                bail!("Canister '{}' is a remote canister on network '{}', and cannot be installed from here.", canister, &network.name)
            }

            let canister_id = canister_id_store.get(canister)?;
            let canister_info = CanisterInfo::load(&config, canister, Some(canister_id))?;
            if let Some(wasm_path) = opts.wasm {
                // streamlined version, we can ignore most of the environment
                let mode = mode.context("The install mode cannot be auto when using --wasm")?;
                install_canister(
                    env,
                    &mut canister_id_store,
                    canister_id,
                    &canister_info,
                    Some(&wasm_path),
                    argument_from_cli,
                    arg_type,
                    Some(mode),
                    call_sender,
                    opts.upgrade_unchanged,
                    None,
                    opts.yes,
                    None,
                    opts.no_asset_upgrade,
                )
                .await
                .map_err(Into::into)
            } else {
                install_canister(
                    env,
                    &mut canister_id_store,
                    canister_id,
                    &canister_info,
                    None,
                    argument_from_cli,
                    arg_type,
                    mode,
                    call_sender,
                    opts.upgrade_unchanged,
                    None,
                    opts.yes,
                    env_file.as_deref(),
                    opts.no_asset_upgrade,
                )
                .await
                .map_err(Into::into)
            }
        }
    } else if opts.all {
        // Install all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                if pull_canisters_in_config.contains_key(canister) {
                    continue;
                }
                if config_interface.is_remote_canister(canister, &network.name)? {
                    info!(
                        env.get_logger(),
                        "Skipping canister '{}' because it is remote for network '{}'",
                        canister,
                        &network.name,
                    );
                    continue;
                }

                let canister_id = canister_id_store.get(canister)?;
                let canister_info = CanisterInfo::load(&config, canister, Some(canister_id))?;
                install_canister(
                    env,
                    &mut canister_id_store,
                    canister_id,
                    &canister_info,
                    None,
                    None,
                    None,
                    mode,
                    call_sender,
                    opts.upgrade_unchanged,
                    None,
                    opts.yes,
                    env_file.as_deref(),
                    opts.no_asset_upgrade,
                )
                .await?;
            }
        }
        if !pull_canisters_in_config.is_empty() {
            info!(env.get_logger(), "There are pull dependencies defined in dfx.json. Please deploy them using `dfx deps deploy`.");
        }
        Ok(())
    } else {
        unreachable!()
    }
}
