use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use console::style;
use ic_agent::CanisterId;
use serde::Deserialize;
use slog::info;
use slog::Logger;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Set of extras that can be specified in the dfx.json.
struct ExternalBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
    /// Where the wasm output will be located.
    wasm: PathBuf,
    /// Where the candid output will be located.
    candid: PathBuf,
    /// A command to run to build this canister. This is optional if the canister
    /// only needs to exist.
    build: Option<String>,
}

impl ExternalBuilderExtra {
    fn try_from(info: &CanisterInfo, pool: &CanisterPool) -> DfxResult<Self> {
        let deps = match info.get_extra_value("dependencies") {
            None => vec![],
            Some(v) => Vec::<String>::deserialize(v).map_err(|_| {
                DfxError::Unknown(String::from("Field 'dependencies' is of the wrong type"))
            })?,
        };
        let dependencies = deps
            .iter()
            .map(|name| {
                pool.get_first_canister_with_name(name)
                    .map(|c| c.canister_id())
                    .map_or_else(
                        || Err(DfxError::UnknownCanisterNamed(name.clone())),
                        DfxResult::Ok,
                    )
            })
            .collect::<DfxResult<Vec<CanisterId>>>()?;

        let wasm = info.get_extra::<PathBuf>("wasm")?;
        let candid = info.get_extra::<PathBuf>("candid")?;
        let build = info.get_extra::<Option<String>>("build")?;

        Ok(ExternalBuilderExtra {
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
pub struct ExternalBuilder {
    logger: Logger,
}

impl ExternalBuilder {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(ExternalBuilder {
            logger: env.get_logger().clone(),
        })
    }
}

impl CanisterBuilder for ExternalBuilder {
    fn supports(&self, info: &CanisterInfo) -> bool {
        info.get_type() == "external"
    }

    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(ExternalBuilderExtra::try_from(info, pool)?.dependencies)
    }

    fn build(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let ExternalBuilderExtra {
            candid,
            wasm,
            build,
            dependencies,
        } = ExternalBuilderExtra::try_from(info, pool)?;

        let canister_id = info.get_canister_id().unwrap();

        if let Some(command) = build {
            info!(
                self.logger,
                r#"{} '{}'"#,
                style("Executing").green().bold(),
                command
            );

            // First separate everything as if it was read from a shell.
            let args = shell_words::split(&command)
                .map_err(|_| DfxError::BuildError(BuildErrorKind::InvalidBuildCommand(command)))?;
            // No commands, noop.
            if args.len() > 0 {
                run_command(args, &canister_id, &candid, dependencies, pool)?;
            }
        }

        Ok(BuildOutput {
            canister_id,
            wasm: WasmBuildOutput::File(wasm),
            idl: IdlBuildOutput::File(candid),
        })
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

    for ref deps in dependencies {
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

    let output = cmd.output().expect("Could not run external tool.");
    if output.status.success() {
        Ok(())
    } else {
        Err(DfxError::BuildError(BuildErrorKind::ExternalToolError(
            output.status.code(),
        )))
    }
}
