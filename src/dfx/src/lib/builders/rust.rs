use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use ic_agent::CanisterId;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;

pub struct RustBuilder {}

impl RustBuilder {
    pub fn new(_env: &dyn Environment) -> DfxResult<Self> {
        Ok(RustBuilder {})
    }
}

impl CanisterBuilder for RustBuilder {
    fn supports(&self, info: &CanisterInfo) -> bool {
        info.get_type() == "rust"
    }

    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        let deps = info.get_extra_value("dependencies");
        let deps = match deps {
            None => vec![],
            Some(v) => Vec::<String>::deserialize(v).map_err(|_| {
                DfxError::Unknown(String::from("Field 'dependencies' is of the wrong type"))
            })?,
        };

        Ok(deps
            .iter()
            .filter_map(|name| {
                pool.get_first_canister_with_name(name)
                    .map(|c| c.canister_id())
            })
            .collect())
    }

    fn build(
        &self,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let canister_id = canister_info.get_canister_id().unwrap();

        let (create_candid, candid_path) = {
            let candid_path = canister_info
                .get_extra::<PathBuf>("candid")
                .map(|candid_path| canister_info.get_workspace_root().join(candid_path));

            match candid_path {
                // Candid must be created. It wasn't specified in the dfx.json.
                Err(_) => {
                    let candid_dir = tempfile::tempdir()?;
                    std::fs::create_dir_all(candid_dir.path())?;
                    let candid_path = candid_dir.path().join("candid.did");
                    Ok((true, candid_path))
                }
                Ok(candid_path) => {
                    if !candid_path.exists() {
                        Err(DfxError::BuildError(BuildErrorKind::CustomError(
                            "IDL file must exist.".to_string(),
                        )))
                    } else {
                        Ok((false, candid_path))
                    }
                }
            }
        }?;

        let output_path = canister_info.get_extra::<PathBuf>("output")?;
        let output_path = canister_info.get_workspace_root().join(output_path);

        // First, run cargo clean to make sure we don't have bad artifacts. Ignore errors.
        let _ = std::process::Command::new("cargo")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("clean")
            .args(&["--target", "wasm32-unknown-unknown"])
            .args(&["--package", canister_info.get_name()])
            .output();

        let mut cargo_cmd = std::process::Command::new("cargo");

        cargo_cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("CANISTER_ID", format!("{}", canister_id))
            .arg("build")
            .args(&["--target", "wasm32-unknown-unknown"])
            .args(&["--package", canister_info.get_name()]);

        if create_candid {
            cargo_cmd.env(
                "CANDID_OUTPUT_PATH",
                format!("{}", candid_path.to_string_lossy()),
            );
        }

        // Add all canister IDs and Candid paths to environment variables so they can be
        // used during build.
        for c in pool.get_canister_list() {
            cargo_cmd.env(
                format!("CANISTER_ID_{}", c.get_name()),
                c.canister_id().to_text(),
            );
            match c.get_build_output() {
                Some(BuildOutput {
                    idl: IdlBuildOutput::File(ref p),
                    ..
                }) => {
                    cargo_cmd.env(
                        format!("CANISTER_CANDID_{}", c.get_name()),
                        p.to_string_lossy().to_string(),
                    );
                }
                None => {}
            }
        }

        // Run the command.
        let output = cargo_cmd.output()?;
        if !output.status.success() {
            return Err(DfxError::BuildError(BuildErrorKind::CompilerError(
                format!("{:?}", cargo_cmd).to_owned(),
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap(),
            )));
        }
        if !output_path.exists() {
            return Err(DfxError::BuildError(BuildErrorKind::CustomError(
                "The output WASM file does not exist.".to_string(),
            )));
        }

        Ok(BuildOutput {
            canister_id,
            idl: IdlBuildOutput::File(candid_path),
            wasm: WasmBuildOutput::File(output_path),
        })
    }
}
