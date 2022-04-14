use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::named_canister;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::read_module_metadata;

use anyhow::{anyhow, bail, Context};
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::{CanisterInstall, InstallMode};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
use openssl::sha::Sha256;
use slog::info;
use std::collections::HashSet;
use std::io::stdin;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_info: &CanisterInfo,
    args: &[u8],
    mode: InstallMode,
    timeout: Duration,
    call_sender: &CallSender,
    installed_module_hash: Option<Vec<u8>>,
) -> DfxResult {
    let network = env.get_network_descriptor().unwrap();
    if !network.is_ic && named_canister::get_ui_canister_id(network).is_none() {
        named_canister::install_ui_canister(env, network, None).await?;
    }

    if mode == InstallMode::Reinstall {
        let msg = format!(
            "You are about to reinstall the {} canister",
            canister_info.get_name()
        ) + r#"
This will OVERWRITE all the data and code in the canister.

YOU WILL LOSE ALL DATA IN THE CANISTER.");

"#;
        ask_for_consent(&msg)?;
    }

    let mgr = ManagementCanister::create(agent);
    let log = env.get_logger();
    let canister_id = canister_info.get_canister_id().context(format!(
        "Cannot find build output for canister '{}'. Did you forget to run `dfx build`?",
        canister_info.get_name().to_owned()
    ))?;
    if matches!(mode, InstallMode::Reinstall | InstallMode::Upgrade) {
        let candid = read_module_metadata(agent, canister_id, "candid:service").await;
        if let Some(candid) = candid {
            use crate::util::check_candid_file;
            let candid_path = canister_info
                .get_output_idl_path()
                .expect("Generated did file not found");
            let deployed_path = candid_path.with_extension("old.did");
            std::fs::write(&deployed_path, candid)?;
            let (mut env, opt_new) = check_candid_file(&candid_path)?;
            let new_type =
                opt_new.expect("Generated did file should contain some service interface");
            let (env2, opt_old) = check_candid_file(&deployed_path)?;
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
    if canister_info.get_type() == "motoko" && matches!(mode, InstallMode::Upgrade) {
        use crate::lib::canister_info::motoko::MotokoCanisterInfo;
        let info = canister_info.as_info::<MotokoCanisterInfo>()?;
        let stable_path = info.get_output_stable_path();
        let deployed_stable_path = stable_path.with_extension("old.most");
        let stable_types = read_module_metadata(agent, canister_id, "motoko:stable-types").await;
        if let Some(stable_types) = stable_types {
            std::fs::write(&deployed_stable_path, stable_types)?;
            let cache = env.get_cache();
            let output = cache
                .get_binary_command("moc")?
                .arg("--stable-compatible")
                .arg(&deployed_stable_path)
                .arg(&stable_path)
                .output()?;
            if !output.status.success() {
                let msg = format!("Stable interface compatibility check failed for canister '{}'.\nUpgrade will either FAIL or LOSE some stable variable data.\n\n", canister_info.get_name()) + &String::from_utf8_lossy(&output.stderr);
                ask_for_consent(&msg)?;
            }
        }
    }

    let mode_str = match mode {
        InstallMode::Install => "Installing",
        InstallMode::Reinstall => "Reinstalling",
        InstallMode::Upgrade => "Upgrading",
    };

    info!(
        log,
        "{} code for canister {}, with canister_id {}",
        mode_str,
        canister_info.get_name(),
        canister_id,
    );

    let wasm_path = canister_info
        .get_output_wasm_path()
        .expect("Cannot get WASM output path.");
    let wasm_module = std::fs::read(wasm_path)?;

    if mode == InstallMode::Upgrade
        && wasm_module_already_installed(&wasm_module, installed_module_hash.as_deref())
    {
        println!(
            "Module hash {} is already installed.",
            hex::encode(installed_module_hash.unwrap())
        );
    } else {
        match call_sender {
            CallSender::SelectedId => {
                let install_builder = mgr
                    .install_code(&canister_id, &wasm_module)
                    .with_raw_arg(args.to_vec())
                    .with_mode(mode);
                install_builder
                    .build()?
                    .call_and_wait(waiter_with_timeout(timeout))
                    .await?;
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
                    .await?;
            }
        }
    }

    if canister_info.get_type() == "assets" {
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
                .await?;
        };

        info!(log, "Uploading assets to asset canister...");
        post_install_store_assets(canister_info, agent, timeout).await?;
    }

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
