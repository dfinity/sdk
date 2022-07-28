use crate::lib::builders::environment_variables;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::named_canister;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::assets::wallet_wasm;
use crate::util::{expiry_duration, read_module_metadata};

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::{CanisterInstall, InstallMode};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
use itertools::Itertools;
use openssl::sha::Sha256;
use slog::info;
use std::collections::HashSet;
use std::io::stdin;
use std::process::{Command, Stdio};
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
#[context("Failed to install wasm module to canister '{}'.", canister_info.get_name())]
pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_id_store: &mut CanisterIdStore,
    canister_info: &CanisterInfo,
    args: impl FnOnce() -> DfxResult<Vec<u8>>,
    mode: InstallMode,
    timeout: Duration,
    call_sender: &CallSender,
    installed_module_hash: Option<Vec<u8>>,
    upgrade_unchanged: bool,
    pool: Option<&CanisterPool>,
) -> DfxResult {
    let log = env.get_logger();
    let network = env.get_network_descriptor();
    if !network.is_ic && named_canister::get_ui_canister_id(canister_id_store).is_none() {
        named_canister::install_ui_canister(env, canister_id_store, None).await?;
    }

    let canister_id = canister_info.get_canister_id()?;
    if matches!(mode, InstallMode::Reinstall | InstallMode::Upgrade) {
        let candid = read_module_metadata(agent, canister_id, "candid:service").await;
        if let Some(candid) = candid {
            use crate::util::check_candid_file;
            let candid_path = canister_info
                .get_output_idl_path()
                .expect("Generated did file not found");
            let deployed_path = candid_path.with_extension("old.did");
            std::fs::write(&deployed_path, candid).with_context(|| {
                format!(
                    "Failed to write candid to {}.",
                    deployed_path.to_string_lossy()
                )
            })?;
            let (mut env, opt_new) =
                check_candid_file(&candid_path).context("Checking generated did file.")?;
            let new_type =
                opt_new.expect("Generated did file should contain some service interface");
            let (env2, opt_old) =
                check_candid_file(&deployed_path).context("Checking old candid file.")?;
            let old_type =
                opt_old.expect("Deployed did file should contain some service interface");
            let mut gamma = HashSet::new();
            let old_type = env.merge_type(env2, old_type);
            let result = candid::types::subtype::subtype(&mut gamma, &env, &new_type, &old_type);
            if let Err(err) = result {
                let msg = format!("Candid interface compatibility check failed for canister '{}'.\nYou are making a BREAKING change. Other canisters or frontend clients relying on your canister may stop working.\n\n", canister_info.get_name()) + &err.to_string();
                ask_for_consent(&msg)?;
            }
        }
    }
    if canister_info.is_motoko() && matches!(mode, InstallMode::Upgrade) {
        use crate::lib::canister_info::motoko::MotokoCanisterInfo;
        let info = canister_info.as_info::<MotokoCanisterInfo>()?;
        let stable_path = info.get_output_stable_path();
        let deployed_stable_path = stable_path.with_extension("old.most");
        let stable_types = read_module_metadata(agent, canister_id, "motoko:stable-types").await;
        if let Some(stable_types) = stable_types {
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
                .arg(&stable_path)
                .output()
                .context("Failed to run 'moc'.")?;
            if !output.status.success() {
                let msg = format!("Stable interface compatibility check failed for canister '{}'.\nUpgrade will either FAIL or LOSE some stable variable data.\n\n", canister_info.get_name()) + &String::from_utf8_lossy(&output.stderr);
                ask_for_consent(&msg)?;
            }
        }
    }

    let wasm_path = canister_info.get_build_wasm_path();
    let wasm_module = std::fs::read(&wasm_path)
        .with_context(|| format!("Failed to read {}.", wasm_path.to_string_lossy()))?;

    if mode == InstallMode::Upgrade
        && wasm_module_already_installed(&wasm_module, installed_module_hash.as_deref())
        && !upgrade_unchanged
    {
        println!(
            "Module hash {} is already installed.",
            hex::encode(installed_module_hash.unwrap())
        );
    } else {
        install_canister_wasm(
            env,
            agent,
            canister_id,
            Some(canister_info.get_name()),
            &args()?,
            mode,
            timeout,
            call_sender,
            wasm_module,
        )
        .await?;
    }

    if canister_info.is_assets() {
        if let CallSender::Wallet(wallet_id) = call_sender {
            let wallet = Identity::build_wallet_canister(*wallet_id, env).await?;
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
                .call_and_wait(waiter_with_timeout(timeout))
                .await
                .context("Failed to authorize your principal with the canister. You can still control the canister by using your wallet with the --wallet flag.")?;
        };

        info!(log, "Uploading assets to asset canister...");
        post_install_store_assets(canister_info, agent, timeout).await?;
    }

    if !canister_info.get_post_install().is_empty() {
        run_post_install_tasks(env, canister_info, network, pool)?;
    }

    Ok(())
}

#[context("Failed to run post-install tasks")]
fn run_post_install_tasks(
    env: &dyn Environment,
    canister: &CanisterInfo,
    network: &NetworkDescriptor,
    pool: Option<&CanisterPool>,
) -> DfxResult {
    let tmp;
    let pool = match pool {
        Some(pool) => pool,
        None => {
            tmp = env
                .get_config_or_anyhow()?
                .get_config()
                .get_canister_names_with_dependencies(Some(canister.get_name()))
                .and_then(|deps| CanisterPool::load(env, false, &deps))
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
        run_post_install_task(canister, task, network, pool, &dependencies)?;
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
) -> DfxResult {
    let cwd = canister.get_workspace_root();
    let words = shell_words::split(task)
        .with_context(|| format!("Error interpreting post-install task `{task}`"))?;
    let canonicalized = cwd
        .join(&words[0])
        .canonicalize()
        .or_else(|_| which::which(&words[0]))
        .map_err(|_| anyhow!("Cannot find command or file {}", &words[0]))?;
    let mut command = Command::new(&canonicalized);
    command.args(&words[1..]);
    let vars = environment_variables(canister, &network.name, pool, dependencies);
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

#[allow(clippy::too_many_arguments)]
#[context("Failed to install wasm in canister '{}'.", canister_id)]
pub async fn install_canister_wasm(
    env: &dyn Environment,
    agent: &Agent,
    canister_id: Principal,
    canister_name: Option<&str>,
    args: &[u8],
    mode: InstallMode,
    timeout: Duration,
    call_sender: &CallSender,
    wasm_module: Vec<u8>,
) -> DfxResult {
    let log = env.get_logger();
    let mgr = ManagementCanister::create(agent);
    if mode == InstallMode::Reinstall {
        let msg = if let Some(name) = canister_name {
            format!("You are about to reinstall the {name} canister")
        } else {
            format!("You are about to reinstall the canister {canister_id}")
        } + r#"
This will OVERWRITE all the data and code in the canister.

YOU WILL LOSE ALL DATA IN THE CANISTER.");

"#;
        ask_for_consent(&msg)?;
    }
    let mode_str = match mode {
        InstallMode::Install => "Installing",
        InstallMode::Reinstall => "Reinstalling",
        InstallMode::Upgrade => "Upgrading",
    };
    if let Some(name) = canister_name {
        info!(
            log,
            "{mode_str} code for canister {name}, with canister ID {canister_id}",
        );
    } else {
        info!(log, "{mode_str} code for canister {canister_id}");
    }
    match call_sender {
        CallSender::SelectedId => {
            let install_builder = mgr
                .install_code(&canister_id, &wasm_module)
                .with_raw_arg(args.to_vec())
                .with_mode(mode);
            install_builder
                .build()
                .context("Failed to build call sender.")?
                .call_and_wait(waiter_with_timeout(timeout))
                .await
                .context("Failed to install wasm.")?;
        }
        CallSender::Wallet(wallet_id) => {
            let wallet = Identity::build_wallet_canister(*wallet_id, env).await?;
            let install_args = CanisterInstall {
                mode,
                canister_id,
                wasm_module,
                arg: args.to_vec(),
            };
            wallet
                .call(
                    *mgr.canister_id_(),
                    "install_code",
                    Argument::from_candid((install_args,)),
                    0,
                )
                .call_and_wait(waiter_with_timeout(timeout))
                .await
                .context("Failed during wasm installation call.")?;
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
    let mgmt = ManagementCanister::create(agent);
    let wasm = wallet_wasm(env.get_logger())?;
    mgmt.install_code(&id, &wasm)
        .with_mode(mode)
        .call_and_wait(waiter_with_timeout(expiry_duration() * 2))
        .await
        .context("Failed to install wallet wasm.")?;
    let wallet = Identity::build_wallet_canister(id, env).await?;
    wallet
        .wallet_store_wallet_wasm(wasm)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("Failed to store wallet wasm in container.")?;
    Ok(())
}

fn wasm_module_already_installed(
    wasm_to_install: &[u8],
    installed_module_hash: Option<&[u8]>,
) -> bool {
    if let Some(installed_module_hash) = installed_module_hash {
        let mut sha256 = Sha256::new();
        sha256.update(wasm_to_install);
        let installing_module_hash = sha256.finish();
        installed_module_hash == installing_module_hash
    } else {
        false
    }
}

fn ask_for_consent(message: &str) -> DfxResult {
    eprintln!("WARNING!");
    eprintln!("{}", message);
    eprintln!("Do you want to proceed? yes/No");
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .map_err(|err| anyhow!(err))
        .context("Unable to read input")?;
    let input_string = input_string.trim_end();
    if input_string != "yes" {
        bail!("Refusing to install canister without approval");
    }
    Ok(())
}
