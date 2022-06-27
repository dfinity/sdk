use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;

use anyhow::Context;
use fn_error_context::context;
use ic_agent::Agent;
use std::time::Duration;

#[context("Failed to store assets in canister '{}'.", info.get_name())]
pub async fn post_install_store_assets(
    info: &CanisterInfo,
    agent: &Agent,
    timeout: Duration,
) -> DfxResult {
    let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
    let output_assets_path = assets_canister_info.get_output_assets_path();

    let canister_id = info.get_canister_id().expect("Could not find canister ID.");

    let canister = ic_utils::Canister::builder()
        .with_agent(agent)
        .with_canister_id(canister_id)
        .build()
        .context("Failed to build asset canister caller.")?;

    ic_asset::sync(&canister, output_assets_path, timeout)
        .await
        .with_context(|| {
            format!(
                "Failed asset sync with canister {}.",
                canister.canister_id_()
            )
        })?;

    Ok(())
}
