use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use ic_agent::CanisterId;
use std::path::PathBuf;

pub struct RustBuilder {}

impl RustBuilder {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(RustBuilder {})
    }
}

impl CanisterBuilder for RustBuilder {
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        // We don't detect dependencies yet.
        Ok(vec![])
    }

    fn supported_canister_types(&self) -> &[&str] {
        &["rust"]
    }

    fn build(
        &self,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let extras = canister_info.get_metadata();
        let candid_path = extras
            .get("candid")
            .ok_or_else(|| BuildErrorKind::CustomError("Key 'candid' is missing.".to_string()))?
            .as_str()
            .ok_or_else(|| {
                BuildErrorKind::CustomError("Key 'candid' needs to be a string.".to_string())
            })?;

        let candid_path = PathBuf::from(candid_path);
        if !candid_path.exists() {
            return Err(DfxError::BuildError(BuildErrorKind::CustomError(
                "IDL file must exist.".to_string(),
            )));
        }

        let output_path = extras
            .get("output")
            .ok_or_else(|| BuildErrorKind::CustomError("Key 'output' is missing.".to_string()))?
            .as_str()
            .ok_or_else(|| {
                BuildErrorKind::CustomError("Key 'output' needs to be a string.".to_string())
            })?;
        let output_path = PathBuf::from(output_path);

        let mut cargo_cmd = std::process::Command::new("cargo")
            .env(
                "CANISTER_ID",
                format!("{}", canister_info.get_canister_id().unwrap()),
            )
            .arg("build")
            .args(&["--target", "wasm32-unknown-unknown"]);

        // Add all canister IDs to environment variables so they can be used during build.
        pool.get_canister_list().iter().for_each(|c| {
            let cid = c.canister_id();
            cargo_cmd = cargo_cmd.env(format!("CANISTER_ID_{}", c.get_name()), cid.to_text());
        });

        cargo_cmd.output()?;

        Ok(BuildOutput {
            canister_id,
            idl: IdlBuildOutput::File(candid_path),
            wasm: WasmBuildOutput::File(output_path),
        })
    }
}
