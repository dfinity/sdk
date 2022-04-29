//! Named canister module.
//!
//! Contains the Candid UI canister for now
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util;
use crate::util::expiry_duration;

use anyhow::{anyhow, Context};
use ic_types::Principal;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::io::Read;

const UI_CANISTER: &str = "__Candid_UI";

pub async fn install_ui_canister(
    env: &dyn Environment,
    network: &NetworkDescriptor,
    some_canister_id: Option<Principal>,
) -> DfxResult<Principal> {
    let mut id_store = CanisterIdStore::for_network(network).with_context(|| {
        format!(
            "Failed to setup canister id store for network {}.",
            network.name
        )
    })?;
    if id_store.find(UI_CANISTER).is_some() {
        return Err(anyhow!(
            "UI canister already installed on {} network",
            network.name
        ));
    }
    fetch_root_key_if_needed(env)
        .await
        .context("Failed to fetch root key.")?;
    let mgr = ManagementCanister::create(
        env.get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
    );
    info!(
        env.get_logger(),
        "Creating UI canister on the {} network.", network.name
    );
    let mut canister_assets =
        util::assets::ui_canister().context("Failed to get ui canister assets.")?;
    let mut wasm = Vec::new();
    for file in canister_assets
        .entries()
        .context("Failed to get ui canister asset entries.")?
    {
        let mut file = file.context("Failed to examine archive entry.")?;
        if file
            .header()
            .path()
            .context("Failed to get archive entry path.")?
            .ends_with("ui.wasm")
        {
            file.read_to_end(&mut wasm)
                .context("Failed to read wasm.")?;
        }
    }
    let canister_id = match some_canister_id {
        Some(id) => id,
        None => {
            mgr.create_canister()
                .as_provisional_create_with_amount(None)
                .call_and_wait(waiter_with_timeout(expiry_duration()))
                .await
                .context("Create canister call failed.")?
                .0
        }
    };
    mgr.install_code(&canister_id, wasm.as_slice())
        .with_mode(InstallMode::Install)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("Install wasm call failed.")?;
    id_store
        .add(UI_CANISTER, canister_id.to_text())
        .with_context(|| {
            format!(
                "Failed to add canister with name {} and id {} to canister id store.",
                UI_CANISTER,
                canister_id.to_text()
            )
        })?;
    info!(
        env.get_logger(),
        "The UI canister on the \"{}\" network is \"{}\"",
        network.name,
        canister_id.to_text()
    );
    Ok(canister_id)
}
pub fn get_ui_canister_id(network: &NetworkDescriptor) -> Option<Principal> {
    let id_store = CanisterIdStore::for_network(network).ok()?;
    id_store.find(UI_CANISTER)
}
