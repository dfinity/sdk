use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::install_canister::install_wallet;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::read_state_tree_canister_module_hash;
use anyhow::bail;
use clap::Parser;
use dfx_core::identity::wallet::wallet_canister_id;
use ic_utils::interfaces::management_canister::builders::InstallMode;

/// Upgrade the wallet's Wasm module to the current Wasm bundled with DFX.
#[derive(Parser)]
pub struct UpgradeOpts {}

pub async fn exec(env: &dyn Environment, _opts: UpgradeOpts) -> DfxResult {
    let identity_name = env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();

    // Network descriptor will always be set.
    let network = env.get_network_descriptor();

    let canister_id = if let Some(principal) = wallet_canister_id(network, &identity_name)? {
        principal
    } else {
        bail!(
            "There is no wallet defined for identity '{}' on network '{}'.  Nothing to do.",
            identity_name,
            &network.name
        );
    };

    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;
    if read_state_tree_canister_module_hash(agent, canister_id)
        .await?
        .is_none()
    {
        bail!("The cycles wallet canister is empty. Try running `dfx identity deploy-wallet` to install code for the cycles wallet in this canister.")
    }

    let agent = env.get_agent();

    install_wallet(
        env,
        agent,
        canister_id,
        InstallMode::Upgrade {
            skip_pre_upgrade: Some(false),
        },
    )
    .await?;

    println!("Upgraded the wallet wasm module.");
    Ok(())
}
