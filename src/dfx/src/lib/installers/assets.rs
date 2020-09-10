use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;
use crate::lib::waiter::create_waiter;
use candid::Encode;
use ic_agent::Agent;
use std::path::Path;
use walkdir::WalkDir;

pub async fn post_install_store_assets(
    info: &CanisterInfo,
    agent: &Agent,
    valid_until: u64,
) -> DfxResult {
    let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
    let output_assets_path = assets_canister_info.get_output_assets_path();

    let walker = WalkDir::new(output_assets_path).into_iter();
    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            let source = entry.path();
            let relative: &Path = source
                .strip_prefix(output_assets_path)
                .expect("cannot strip prefix");
            let content = &std::fs::read(&source)?;
            let path = relative.to_string_lossy().to_string();
            let blob = candid::Encode!(&path, &content)?;

            let canister_id = info.get_canister_id().expect("Could not find canister ID.");
            let method_name = String::from("store");
            agent
                .update(&canister_id, &method_name)
                .with_arg(&blob)
                .with_expiry(valid_until)
                .call_and_wait(create_waiter())
                .await?;
        }
    }
    Ok(())
}
