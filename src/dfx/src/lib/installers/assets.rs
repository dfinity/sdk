use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;
use candid::Encode;
use delay::Delay;
use ic_agent::Agent;
use ic_agent::Blob;
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;

const RETRY_PAUSE: Duration = Duration::from_millis(100);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub fn create_waiter() -> Delay {
    Delay::builder()
        .throttle(RETRY_PAUSE)
        .timeout(REQUEST_TIMEOUT)
        .build()
}

pub async fn post_install_store_assets(info: &CanisterInfo, agent: &Agent) -> DfxResult {
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
            let content: String = base64::encode(&std::fs::read(&source)?);
            let path = relative.to_string_lossy().to_string();
            let parameter = Encode!(&path, &content)?;
            let blob = Blob::from(&parameter);
            let canister_id = info.get_canister_id().expect("Could not find canister ID.");
            let method_name = String::from("store");
            agent
                .call_and_wait(&canister_id, &method_name, &blob, create_waiter())
                .await?;
        }
    }
    Ok(())
}
