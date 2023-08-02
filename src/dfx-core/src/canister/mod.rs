use crate::{
    cli::ask_for_consent,
    error::canister::{CanisterBuilderError, CanisterInstallError},
    identity::CallSender,
};
use candid::Principal;
use ic_agent::Agent;
use ic_utils::{
    call::AsyncCall,
    interfaces::{
        management_canister::builders::{CanisterInstall, InstallMode},
        ManagementCanister, WalletCanister,
    },
    Argument,
};
use slog::{info, Logger};

pub async fn build_wallet_canister(
    id: Principal,
    agent: &Agent,
) -> Result<WalletCanister<'_>, CanisterBuilderError> {
    WalletCanister::from_canister(
        ic_utils::Canister::builder()
            .with_agent(agent)
            .with_canister_id(id)
            .build()
            .unwrap(),
    )
    .await
    .map_err(CanisterBuilderError::WalletCanisterCaller)
}

pub async fn install_canister_wasm(
    agent: &Agent,
    canister_id: Principal,
    canister_name: Option<&str>,
    args: &[u8],
    mode: InstallMode,
    call_sender: &CallSender,
    wasm_module: Vec<u8>,
    skip_consent: bool,
    logger: &Logger,
) -> Result<(), CanisterInstallError> {
    let mgr = ManagementCanister::create(agent);
    if !skip_consent && mode == InstallMode::Reinstall {
        let msg = if let Some(name) = canister_name {
            format!("You are about to reinstall the {name} canister")
        } else {
            format!("You are about to reinstall the canister {canister_id}")
        } + r#"
This will OVERWRITE all the data and code in the canister.

YOU WILL LOSE ALL DATA IN THE CANISTER.

"#;
        ask_for_consent(&msg).map_err(CanisterInstallError::UserConsent)?;
    }
    let mode_str = match mode {
        InstallMode::Install => "Installing",
        InstallMode::Reinstall => "Reinstalling",
        InstallMode::Upgrade => "Upgrading",
    };
    if let Some(name) = canister_name {
        info!(
            logger,
            "{mode_str} code for canister {name}, with canister ID {canister_id}",
        );
    } else {
        info!(logger, "{mode_str} code for canister {canister_id}");
    }
    match call_sender {
        CallSender::SelectedId => {
            let install_builder = mgr
                .install_code(&canister_id, &wasm_module)
                .with_raw_arg(args.to_vec())
                .with_mode(mode);
            install_builder
                .build()
                .map_err(CanisterBuilderError::CallSenderBuildError)?
                .call_and_wait()
                .await
                .map_err(CanisterInstallError::InstallWasmError)
        }
        CallSender::Wallet(wallet_id) => {
            let wallet = build_wallet_canister(*wallet_id, agent).await?;
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
                .call_and_wait()
                .await
                .map_err(CanisterInstallError::InstallWasmError)
        }
    }
}
