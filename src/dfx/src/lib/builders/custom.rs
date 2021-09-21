use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;

use anyhow::{anyhow, Context};
use console::style;
use ic_types::principal::Principal as CanisterId;
use serde::Deserialize;
use slog::info;
use slog::Logger;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Set of extras that can be specified in the dfx.json.
struct CustomBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
    /// Where the wasm output will be located.
    wasm: PathBuf,
    /// Where the candid output will be located.
    candid: PathBuf,
    /// A command to run to build this canister. This is optional if the canister
    /// only needs to exist.
    build: Vec<String>,
}

impl CustomBuilderExtra {
    fn try_from(info: &CanisterInfo, pool: &CanisterPool) -> DfxResult<Self> {
        let deps = match info.get_extra_value("dependencies") {
            None => vec![],
            Some(v) => Vec::<String>::deserialize(v)
                .map_err(|_| anyhow!("Field 'dependencies' is of the wrong type."))?,
        };
        let dependencies = deps
            .iter()
            .map(|name| {
                pool.get_first_canister_with_name(name)
                    .map(|c| c.canister_id())
                    .map_or_else(
                        || Err(anyhow!("A canister with the name '{}' was not found in the current project.", name.clone())),
                        DfxResult::Ok,
                    )
            })
            .collect::<DfxResult<Vec<CanisterId>>>()?;

        let wasm = info
            .get_output_wasm_path()
            .expect("Missing wasm key in JSON.");
        let candid = info
            .get_output_idl_path()
            .expect("Missing candid key in JSON.");
        let build = if let Some(json) = info.get_extra_value("build") {
            if let Ok(s) = String::deserialize(json.clone()) {
                vec![s]
            } else {
                Vec::<String>::deserialize(json)?
            }
        } else {
            vec![]
        };

        Ok(CustomBuilderExtra {
            dependencies,
            wasm,
            candid,
            build,
        })
    }
}

/// A Builder for a WASM type canister, which has an optional build step.
/// This will set environment variables for the external tool;
///   `CANISTER_ID`     => Its own canister ID (in textual format).
///   `CANDID_PATH`     => Its own candid path.
///   `CANISTER_ID_{}`  => The canister ID of all dependencies. `{}` is replaced by the name.
///   `CANDID_{}`       => The candid path of all dependencies. `{}` is replaced by the name.
pub struct CustomBuilder {
    logger: Logger,
}

impl CustomBuilder {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(CustomBuilder {
            logger: env.get_logger().clone(),
        })
    }
}

impl CanisterBuilder for CustomBuilder {
    fn supports(&self, info: &CanisterInfo) -> bool {
        info.get_type() == "custom"
    }

    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(CustomBuilderExtra::try_from(info, pool)?.dependencies)
    }

    fn build(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let CustomBuilderExtra {
            candid,
            wasm,
            build,
            dependencies,
        } = CustomBuilderExtra::try_from(info, pool)?;

        let canister_id = info.get_canister_id().unwrap();

        for command in build {
            info!(
                self.logger,
                r#"{} '{}'"#,
                style("Executing").green().bold(),
                command
            );

            // First separate everything as if it was read from a shell.
            let args = shell_words::split(&command)
                .context(format!("Cannot parse command '{}'.", command))?;
            // No commands, noop.
            if !args.is_empty() {
                run_command(args, &canister_id, &candid, dependencies.clone(), pool)?;
            }
        }

        Ok(BuildOutput {
            canister_id,
            wasm: WasmBuildOutput::File(wasm),
            idl: IdlBuildOutput::File(candid),
        })
    }

    fn generate_idl(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        let generate_output_dir = &info
            .get_declarations_config()
            .output
            .as_ref()
            .context("output here must not be None")?;

        std::fs::create_dir_all(generate_output_dir)?;

        let output_idl_path = generate_output_dir
            .join(info.get_name())
            .with_extension("did");

        // get the path to candid file
        let CustomBuilderExtra { candid, .. } = CustomBuilderExtra::try_from(info, pool)?;

        std::fs::copy(&candid, &output_idl_path)?;

        Ok(output_idl_path)
    }
}

fn run_command(
    args: Vec<String>,
    canister_id: &CanisterId,
    candid: &Path,
    dependencies: Vec<CanisterId>,
    pool: &CanisterPool,
) -> DfxResult<()> {
    let (command_name, arguments) = args.split_first().unwrap();

    let mut cmd = std::process::Command::new(command_name);

    cmd.args(arguments)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("CANISTER_ID", canister_id.to_text())
        .env("CANISTER_CANDID_PATH", candid.as_os_str());

    for deps in &dependencies {
        let canister = pool.get_canister(deps).unwrap();
        cmd.env(
            format!("CANISTER_ID_{}", canister.get_name()),
            deps.to_text(),
        );
        if let Some(output) = canister.get_build_output() {
            let candid_path = match &output.idl {
                IdlBuildOutput::File(p) => p.as_os_str(),
            };

            cmd.env(
                format!("CANISTER_CANDID_{}", canister.get_name()),
                candid_path,
            );
        }
    }

    let output = cmd.output().expect("Could not run custom tool.");
    if output.status.success() {
        Ok(())
    } else {
        Err(DfxError::new(BuildError::CustomToolError(
            output.status.code(),
        )))
    }
}
