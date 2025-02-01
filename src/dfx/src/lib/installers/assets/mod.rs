use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::progress::EnvAssetSyncProgressRenderer;
use anyhow::Context;
use fn_error_context::context;
use ic_agent::Agent;
use std::path::Path;

#[context("Failed to store assets in canister '{}'.", info.get_name())]
pub async fn post_install_store_assets(
    env: &dyn Environment,
    info: &CanisterInfo,
    agent: &Agent,
) -> DfxResult {
    let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
    let source_paths = assets_canister_info.get_source_paths();
    let source_paths: Vec<&Path> = source_paths.iter().map(|p| p.as_path()).collect::<_>();

    let canister_id = info
        .get_canister_id()
        .context("Could not find canister ID.")?;

    let canister = ic_utils::Canister::builder()
        .with_agent(agent)
        .with_canister_id(canister_id)
        .build()
        .context("Failed to build asset canister caller.")?;

    let progress = EnvAssetSyncProgressRenderer::new(env);

    ic_asset::sync(
        &canister,
        &source_paths,
        false,
        env.get_logger(),
        Some(&progress),
    )
    .await
    .with_context(|| {
        format!(
            "Failed asset sync with canister {}.",
            canister.canister_id_()
        )
    })
}

#[context("Failed to store assets in canister '{}'.", info.get_name())]
pub async fn prepare_assets_for_proposal(
    info: &CanisterInfo,
    agent: &Agent,
    env: &dyn Environment,
) -> DfxResult {
    let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
    let source_paths = assets_canister_info.get_source_paths();
    let source_paths: Vec<&Path> = source_paths.iter().map(|p| p.as_path()).collect::<_>();

    let canister_id = info
        .get_canister_id()
        .context("Could not find canister ID.")?;

    let canister = ic_utils::Canister::builder()
        .with_agent(agent)
        .with_canister_id(canister_id)
        .build()
        .context("Failed to build asset canister caller.")?;

    let r = EnvAssetSyncProgressRenderer::new(env);

    ic_asset::prepare_sync_for_proposal(&canister, &source_paths, env.get_logger(), Some(&r))
        .await
        .with_context(|| {
            format!(
                "Failed asset sync with canister {}.",
                canister.canister_id_()
            )
        })?;

    Ok(())
}
