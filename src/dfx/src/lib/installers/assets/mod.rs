use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;

use ic_agent::Agent;
use std::time::Duration;

pub async fn post_install_store_assets(
    info: &CanisterInfo,
    agent: &Agent,
    timeout: Duration,
) -> DfxResult {
    let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
    let output_assets_path = assets_canister_info.get_output_assets_path();

    let canister_id = info.get_canister_id().expect("Could not find canister ID.");

    ic_asset::sync(agent, output_assets_path, &canister_id, timeout).await?;

    Ok(())
}
