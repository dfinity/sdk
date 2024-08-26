use crate::lib::builders::get_and_write_environment_variables;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::models::canister::CanisterPool;
use crate::lib::named_canister;
use crate::lib::operations::canister::all_project_canisters_with_ids;
use crate::lib::operations::canister::motoko_playground::authorize_asset_uploader;
use crate::lib::state_tree::canister_info::read_state_tree_canister_module_hash;
use crate::util::assets::wallet_wasm;
use crate::util::{blob_from_arguments, get_candid_init_type, read_module_metadata};
use anyhow::{anyhow, bail, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::Principal;
use dfx_core::canister::{build_wallet_canister, install_canister_wasm, install_mode_to_prompt};
use dfx_core::cli::ask_for_consent;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::management_canister::builders::{
    CanisterUpgradeOptions, InstallMode, WasmMemoryPersistence,
};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use slog::{debug, info, warn};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::motoko_playground::playground_install_code;

#[context("Failed to install wasm module to canister '{}'.", canister_info.get_name())]
pub async fn install_canister(
    env: &dyn Environment,
    canister_id_store: &mut CanisterIdStore,
    canister_id: Principal,
    canister_info: &CanisterInfo,
    wasm_path_override: Option<&Path>,
    argument_from_cli: Option<&str>,
    argument_type_from_cli: Option<&str>,
    mode: Option<InstallMode>,
    call_sender: &CallSender,
    upgrade_unchanged: bool,
    pool: Option<&CanisterPool>,
    skip_consent: bool,
    env_file: Option<&Path>,
    no_asset_upgrade: bool,
    always_assist: bool,
) -> DfxResult {
    let log = env.get_logger();
    let agent = env.get_agent();
    let network = env.get_network_descriptor();
    if !network.is_ic && named_canister::get_ui_canister_id(canister_id_store).is_none() {
        named_canister::install_ui_canister(env, canister_id_store, None).await?;
    }
    let installed_module_hash = read_state_tree_canister_module_hash(agent, canister_id).await?;
    debug!(
        log,
        "Previously installed module hash: {:?}",
        installed_module_hash.as_ref().map(hex::encode)
    );
    let wasm_memory_persistence =
        read_module_metadata(agent, canister_id, "enhanced-orthogonal-persistence")
            .await
            .map(|_| WasmMemoryPersistence::Keep);
    let mode = mode.unwrap_or_else(|| {
        if installed_module_hash.is_some() {
            InstallMode::Upgrade(Some(CanisterUpgradeOptions {
                wasm_memory_persistence,
                skip_pre_upgrade: None,
            }))
        } else {
            InstallMode::Install
        }
    });
    let mode_str = install_mode_to_prompt(&mode);
    let canister_name = canister_info.get_name();
    info!(
        log,
        "{mode_str} code for canister {canister_name}, with canister ID {canister_id}",
    );
    if !skip_consent && matches!(mode, InstallMode::Reinstall | InstallMode::Upgrade { .. }) {
        let candid = read_module_metadata(agent, canister_id, "candid:service").await;
        if let Some(candid) = &candid {
            match check_candid_compatibility(canister_info, candid) {
                Ok(None) => (),
                Ok(Some(err)) => {
                    let msg = format!("Candid interface compatibility check failed for canister '{}'.\nYou are making a BREAKING change. Other canisters or frontend clients relying on your canister may stop working.\n\n", canister_info.get_name()) + &err;
                    ask_for_consent(&msg)?;
                }
                Err(e) => {
                    let msg = format!("An error occurred during Candid interface compatibility check for canister '{}'.\n\n", canister_info.get_name()) + &e.to_string();
                    ask_for_consent(&msg)?;
                }
            }
        }
    }
    if !skip_consent && canister_info.is_motoko() && matches!(mode, InstallMode::Upgrade { .. }) {
        let stable_types = read_module_metadata(agent, canister_id, "motoko:stable-types").await;
        if let Some(stable_types) = &stable_types {
            match check_stable_compatibility(canister_info, env, stable_types) {
                Ok(None) => (),
                Ok(Some(err)) => {
                    let msg = format!("Stable interface compatibility check failed for canister '{}'.\nUpgrade will either FAIL or LOSE some stable variable data.\n\n", canister_info.get_name()) + &err;
                    ask_for_consent(&msg)?;
                }
                Err(e) => {
                    let msg = format!("An error occurred during stable interface compatibility check for canister '{}'.\n\n", canister_info.get_name()) + &e.to_string();
                    ask_for_consent(&msg)?;
                }
            }
        }
    }

    let wasm_path: PathBuf = if let Some(wasm_override) = wasm_path_override {
        wasm_override.into()
    } else {
        let build_wasm_path = canister_info.get_build_wasm_path();
        if !build_wasm_path.exists() {
            bail!("The canister must be built before install. Please run `dfx build`.");
        }
        build_wasm_path
    };
    let wasm_module = dfx_core::fs::read(&wasm_path)?;
    let new_hash = Sha256::digest(&wasm_module);
    debug!(log, "New wasm module hash: {}", hex::encode(new_hash));

    if matches!(mode, InstallMode::Upgrade { .. })
        && matches!(&installed_module_hash, Some(old_hash) if old_hash[..] == new_hash[..])
        && !upgrade_unchanged
    {
        println!(
            "Module hash {} is already installed.",
            hex::encode(installed_module_hash.as_ref().unwrap())
        );
    } else if !(canister_info.is_assets() && no_asset_upgrade) {
        let idl_path = canister_info.get_constructor_idl_path();
        let init_type = if wasm_path_override.is_some() {
            None
        } else {
            get_candid_init_type(&idl_path)
        };

        // The argument and argument_type from the CLI take precedence over the dfx.json configuration.
        let argument_from_json = canister_info.get_init_arg()?;
        let (argument, argument_type) = match (argument_from_cli, &argument_from_json) {
            (Some(a_cli), Some(a_json)) => {
                // We want to warn the user when the argument from CLI and json are different.
                // There are two cases to consider:
                // 1. The argument from CLI is in raw format, while the argument from json is always in Candid format.
                // 2. Both arguments are in Candid format, but they are different.
                if argument_type_from_cli == Some("raw") || a_cli != a_json {
                    warn!(
                        log,
                        "Canister '{0}' has init_arg/init_arg_file in dfx.json: {1},
which is different from the one specified in the command line: {2}.
The command line value will be used.",
                        canister_info.get_name(),
                        a_json,
                        a_cli
                    );
                }
                (argument_from_cli, argument_type_from_cli)
            }
            (Some(_), None) => (argument_from_cli, argument_type_from_cli),
            (None, Some(a_json)) => (Some(a_json.as_str()), Some("idl")), // `init_arg` in dfx.json is always in Candid format
            (None, None) => (None, None),
        };
        let install_args = blob_from_arguments(
            Some(env),
            argument,
            None,
            argument_type,
            &init_type,
            true,
            always_assist,
        )?;
        if let Some(timestamp) = canister_id_store.get_timestamp(canister_info.get_name()) {
            let new_timestamp = playground_install_code(
                env,
                canister_id,
                timestamp,
                &install_args,
                &wasm_module,
                mode,
                canister_info.is_assets(),
            )
            .await?;
            canister_id_store.add(
                canister_info.get_name(),
                &canister_id.to_string(),
                Some(new_timestamp),
            )?;
        } else {
            install_canister_wasm(
                agent,
                canister_id,
                Some(canister_info.get_name()),
                &install_args,
                mode,
                call_sender,
                wasm_module,
                skip_consent,
            )
            .await?;
        }
    }

    wait_for_module_hash(
        env,
        agent,
        canister_id,
        installed_module_hash.as_deref(),
        &new_hash,
    )
    .await?;

    if canister_info.is_assets() {
        if let Some(canister_timeout) = canister_id_store.get_timestamp(canister_info.get_name()) {
            // playground installed the code, so playground has to authorize call_sender to upload files
            let uploader_principal = env
                .get_selected_identity_principal()
                .context("Failed to figure out selected identity's principal.")?;
            authorize_asset_uploader(
                env,
                canister_info.get_canister_id()?,
                canister_timeout,
                &uploader_principal,
            )
            .await?;
        }
        if let CallSender::Wallet(wallet_id) = call_sender {
            let wallet = build_wallet_canister(*wallet_id, agent).await?;
            let identity_name = env.get_selected_identity().expect("No selected identity.");
            info!(
                log,
                "Authorizing our identity ({}) to the asset canister...", identity_name
            );
            let self_id = env
                .get_selected_identity_principal()
                .expect("Selected identity not instantiated.");
            // Before storing assets, make sure the DFX principal is in there first.
            wallet
                .call(
                    canister_id,
                    "authorize",
                    Argument::from_candid((self_id,)),
                    0,
                )
                .await
                .context("Failed to authorize your principal with the canister. You can still control the canister by using your wallet with the --wallet flag.")?;
        };

        info!(log, "Uploading assets to asset canister...");
        post_install_store_assets(canister_info, agent, log).await?;
    }
    if !canister_info.get_post_install().is_empty() {
        let config = env.get_config()?;
        run_post_install_tasks(
            env,
            canister_info,
            network,
            pool,
            env_file.or_else(|| config.as_ref()?.get_config().output_env_file.as_deref()),
        )?;
    }

    Ok(())
}

fn check_candid_compatibility(
    canister_info: &CanisterInfo,
    candid: &str,
) -> anyhow::Result<Option<String>> {
    use candid::types::subtype::{subtype_with_config, OptReport};
    use candid_parser::utils::CandidSource;
    let candid_path = canister_info.get_constructor_idl_path();
    let deployed_path = canister_info
        .get_constructor_idl_path()
        .with_extension("old.did");
    std::fs::write(&deployed_path, candid).with_context(|| {
        format!(
            "Failed to write candid to {}.",
            deployed_path.to_string_lossy()
        )
    })?;
    let (mut env, opt_new) = CandidSource::File(&candid_path)
        .load()
        .context("Checking generated did file.")?;
    let new_type = opt_new
        .ok_or_else(|| anyhow!("Generated did file should contain some service interface"))?;
    let (env2, opt_old) = CandidSource::File(&deployed_path)
        .load()
        .context("Checking old candid file.")?;
    let old_type = opt_old
        .ok_or_else(|| anyhow!("Deployed did file should contain some service interface"))?;
    let mut gamma = HashSet::new();
    let old_type = env.merge_type(env2, old_type);
    let result = subtype_with_config(OptReport::Error, &mut gamma, &env, &new_type, &old_type);
    Ok(match result {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    })
}

async fn wait_for_module_hash(
    env: &dyn Environment,
    agent: &Agent,
    canister_id: Principal,
    old_hash: Option<&[u8]>,
    new_hash: &[u8],
) -> DfxResult {
    let mut retry_policy = ExponentialBackoff::default();
    let mut times = 0;
    loop {
        match read_state_tree_canister_module_hash(agent, canister_id).await? {
            Some(reported_hash) => {
                if env.get_network_descriptor().is_playground() {
                    // Playground may modify wasm before installing, therefore we cannot predict what the hash is supposed to be.
                    info!(
                        env.get_logger(),
                        "Something is installed in canister {}. Assuming new code is installed.",
                        canister_id
                    );
                    break;
                }
                if reported_hash[..] == new_hash[..] {
                    break;
                } else if old_hash.map_or(true, |old_hash| old_hash == reported_hash) {
                    times += 1;
                    if times > 3 {
                        info!(
                            env.get_logger(),
                            "Waiting for module change to be reflected in system state tree..."
                        )
                    }
                    let interval = retry_policy.next_backoff()
                            .context("Timed out waiting for the module to update to the new hash in the state tree. \
                                Something may have gone wrong with the upload. \
                                No post-installation tasks have been run, including asset uploads.")?;
                    tokio::time::sleep(interval).await;
                } else {
                    bail!("The reported module hash ({reported}) is neither the existing module ({old}) or the new one ({new}). \
                            It has likely been modified while this command is running. \
                            The state of the canister is unknown. \
                            For this reason, no post-installation tasks have been run, including asset uploads.",
                            old = old_hash.map_or_else(|| "none".to_string(), hex::encode),
                            new = hex::encode(new_hash),
                            reported = hex::encode(reported_hash),
                        )
                }
            }
            None => {
                times += 1;
                if times > 3 {
                    info!(
                        env.get_logger(),
                        "Waiting for module change to be reflected in system state tree..."
                    )
                }
                let interval = retry_policy.next_backoff()
                        .context("Timed out waiting for the module to update to the new hash in the state tree. \
                            Something may have gone wrong with the upload. \
                            No post-installation tasks have been run, including asset uploads.")?;
                tokio::time::sleep(interval).await;
            }
        }
    }
    Ok(())
}

fn check_stable_compatibility(
    canister_info: &CanisterInfo,
    env: &dyn Environment,
    stable_types: &str,
) -> anyhow::Result<Option<String>> {
    use crate::lib::canister_info::motoko::MotokoCanisterInfo;
    let info = canister_info.as_info::<MotokoCanisterInfo>()?;
    let stable_path = info.get_output_stable_path();
    let deployed_stable_path = stable_path.with_extension("old.most");
    std::fs::write(&deployed_stable_path, stable_types).with_context(|| {
        format!(
            "Failed to write stable types to {}.",
            deployed_stable_path.to_string_lossy()
        )
    })?;
    let cache = env.get_cache();
    let output = cache
        .get_binary_command("moc")?
        .arg("--stable-compatible")
        .arg(&deployed_stable_path)
        .arg(stable_path)
        .output()
        .context("Failed to run 'moc'.")?;
    Ok(if !output.status.success() {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        None
    })
}

#[context("Failed to run post-install tasks")]
fn run_post_install_tasks(
    env: &dyn Environment,
    canister: &CanisterInfo,
    network: &NetworkDescriptor,
    pool: Option<&CanisterPool>,
    env_file: Option<&Path>,
) -> DfxResult {
    let tmp;
    let pool = match pool {
        Some(pool) => pool,
        None => {
            let config = env.get_config_or_anyhow()?;
            let canisters_to_load = all_project_canisters_with_ids(env, &config);

            tmp = CanisterPool::load(env, false, &canisters_to_load)
                .context("Error collecting canisters for post-install task")?;
            &tmp
        }
    };
    let dependencies = pool
        .get_canister_list()
        .iter()
        .map(|can| can.canister_id())
        .collect_vec();
    for task in canister.get_post_install() {
        run_post_install_task(canister, task, network, pool, &dependencies, env_file)?;
    }
    Ok(())
}

#[context("Failed to run post-install task {task}")]
fn run_post_install_task(
    canister: &CanisterInfo,
    task: &str,
    network: &NetworkDescriptor,
    pool: &CanisterPool,
    dependencies: &[Principal],
    env_file: Option<&Path>,
) -> DfxResult {
    let cwd = canister.get_workspace_root();
    let words = shell_words::split(task)
        .with_context(|| format!("Error interpreting post-install task `{task}`"))?;
    let canonicalized = dfx_core::fs::canonicalize(&cwd.join(&words[0]))
        .or_else(|_| which::which(&words[0]))
        .map_err(|_| anyhow!("Cannot find command or file {}", &words[0]))?;
    let mut command = Command::new(canonicalized);
    command.args(&words[1..]);
    let vars =
        get_and_write_environment_variables(canister, &network.name, pool, dependencies, env_file)?;
    for (key, val) in vars {
        command.env(&*key, val);
    }
    command
        .current_dir(cwd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let status = command.status()?;
    if !status.success() {
        match status.code() {
            Some(code) => {
                bail!("The post-install task `{task}` failed with exit code {code}")
            }
            None => bail!("The post-install task `{task}` was terminated by a signal"),
        }
    }
    Ok(())
}

pub async fn install_wallet(
    env: &dyn Environment,
    agent: &Agent,
    id: Principal,
    mode: InstallMode,
) -> DfxResult {
    if env.get_network_descriptor().is_playground() {
        bail!("Refusing to install wallet. Wallets do not work for playground networks.");
    }
    let mgmt = ManagementCanister::create(agent);
    let wasm = wallet_wasm(env.get_logger())?;
    mgmt.install_code(&id, &wasm)
        .with_mode(mode)
        .await
        .context("Failed to install wallet wasm.")?;
    wait_for_module_hash(env, agent, id, None, &Sha256::digest(&wasm)).await?;
    let wallet = build_wallet_canister(id, agent).await?;
    wallet
        .wallet_store_wallet_wasm(wasm)
        .await
        .context("Failed to store wallet wasm in container.")?;
    Ok(())
}
