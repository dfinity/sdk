use crate::{
    error::{
        canister::{CanisterBuilderError, CanisterInstallError},
        cli::UserConsent,
    },
    identity::CallSender,
};
use candid::Principal;
use ic_agent::Agent;
use ic_utils::{
    Argument,
    interfaces::{
        ManagementCanister, WalletCanister,
        management_canister::builders::{CanisterInstallMode, InstallCodeArgs},
    },
};

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

pub fn install_mode_to_present_tense(mode: &CanisterInstallMode) -> &'static str {
    match mode {
        CanisterInstallMode::Install => "Installing",
        CanisterInstallMode::Reinstall => "Reinstalling",
        CanisterInstallMode::Upgrade { .. } => "Upgrading",
    }
}

pub fn install_mode_to_past_tense(mode: &CanisterInstallMode) -> &'static str {
    match mode {
        CanisterInstallMode::Install => "Installed",
        CanisterInstallMode::Reinstall => "Reinstalled",
        CanisterInstallMode::Upgrade { .. } => "Upgraded",
    }
}

pub async fn install_canister_wasm(
    agent: &Agent,
    canister_id: Principal,
    canister_name: Option<&str>,
    args: &[u8],
    mode: CanisterInstallMode,
    call_sender: &CallSender,
    wasm_module: Vec<u8>,
    ask_for_consent: impl FnOnce(&str) -> Result<(), UserConsent> + Send,
) -> Result<(), CanisterInstallError> {
    let mgr = ManagementCanister::create(agent);
    if mode == CanisterInstallMode::Reinstall {
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

    match call_sender {
        CallSender::SelectedId => {
            let install_builder = mgr
                .install(&canister_id, &wasm_module)
                .with_raw_arg(args.to_vec())
                .with_mode(mode);
            install_builder
                .await
                .map_err(CanisterInstallError::InstallWasmError)
        }
        CallSender::Impersonate(_) => {
            unreachable!("Impersonating sender when installing canisters is not supported.")
        }
        CallSender::Wallet(wallet_id) => {
            let wallet = build_wallet_canister(*wallet_id, agent).await?;
            let install_args = InstallCodeArgs {
                mode,
                canister_id,
                wasm_module,
                arg: args.to_vec(),
                sender_canister_version: None,
            };
            wallet
                .call(
                    *mgr.canister_id_(),
                    "install_code",
                    Argument::from_candid((install_args,)),
                    0,
                )
                .await
                .map_err(CanisterInstallError::InstallWasmError)
        }
    }
}
