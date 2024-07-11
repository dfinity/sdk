use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use anyhow::{anyhow, bail};
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use std::path::{Path, PathBuf};
use url::Url;

enum CustomFileLocation {
    OutputPath(PathBuf),
    DownloadUrl(Url),
}

pub struct CustomCanisterInfo {
    input_wasm_url: Option<Url>,
    output_wasm_path: PathBuf,
    input_candid_url: Option<Url>,
    build: Vec<String>,
}

impl CustomCanisterInfo {
    pub fn get_input_wasm_url(&self) -> &Option<Url> {
        &self.input_wasm_url
    }
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_input_candid_url(&self) -> &Option<Url> {
        &self.input_candid_url
    }
    pub fn get_build_tasks(&self) -> &[String] {
        &self.build
    }
}

impl CanisterInfoFactory for CustomCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let workspace_root = info.get_workspace_root();
        let (wasm, build, candid) = if let CanisterTypeProperties::Custom {
            wasm,
            build,
            candid,
        } = info.type_specific.clone()
        {
            (wasm, build.into_vec(), candid)
        } else {
            bail!(
                "Attempted to construct a custom canister from a type:{} canister config",
                info.type_specific.name()
            )
        };
        let (input_wasm_url, output_wasm_path) = if let Ok(input_wasm_url) = Url::parse(&wasm) {
            if !build.is_empty() {
                bail!(
                    "Canister '{}' defines its wasm field as a URL, and has a build step.",
                    info.name
                );
            }
            let filename = input_wasm_url.path_segments().ok_or_else(|| {
                anyhow!(
                    "unable to determine path segments for url {}",
                    &input_wasm_url
                )
            })?;
            let filename = filename.last().ok_or_else(|| {
                anyhow!("Unable to determine filename for url {}", &input_wasm_url)
            })?;
            let output_wasm_path = info
                .get_output_root()
                .join(format!("download-{}", filename));
            (Some(input_wasm_url), output_wasm_path)
        } else {
            let output_wasm_path = workspace_root.join(wasm);
            (None, output_wasm_path)
        };
        let input_candid_url = if info.get_remote_candid_if_remote().is_some() {
            None
        } else if let Ok(input_candid_url) = Url::parse(&candid) {
            Some(input_candid_url)
        } else {
            None
        };

        Ok(Self {
            input_wasm_url,
            output_wasm_path,
            input_candid_url,
            build,
        })
    }
}
