use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::custom::CustomCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::util::download_file_to_path;
use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use console::style;
use fn_error_context::context;
use slog::info;
use slog::Logger;
use std::path::PathBuf;
use url::Url;

/// Set of extras that can be specified in the dfx.json.
struct CustomBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
    /// Where to download the wasm from
    input_wasm_url: Option<Url>,
    /// Where the wasm output will be located.
    wasm: PathBuf,
    /// Where to download the candid from
    input_candid_url: Option<Url>,
    /// Where the candid output will be located.
    candid: PathBuf,
    /// A command to run to build this canister. This is optional if the canister
    /// only needs to exist.
    build: Vec<String>,
}

impl CustomBuilderExtra {
    #[context("Failed to create CustomBuilderExtra for canister '{}'.", info.get_name())]
    fn try_from(info: &CanisterInfo, pool: &CanisterPool) -> DfxResult<Self> {
        let dependencies = info.get_dependencies()
            .iter()
            .map(|name| {
                pool.get_first_canister_with_name(name)
                    .map(|c| c.canister_id())
                    .map_or_else(
                        || Err(anyhow!("A canister with the name '{}' was not found in the current project.", name.clone())),
                        DfxResult::Ok,
                    )
            })
            .collect::<DfxResult<Vec<CanisterId>>>().with_context( || format!("Failed to collect dependencies (canister ids) of canister {}.", info.get_name()))?;
        let info = info.as_info::<CustomCanisterInfo>()?;
        let input_wasm_url = info.get_input_wasm_url().to_owned();
        let wasm = info.get_output_wasm_path().to_owned();
        let input_candid_url = info.get_input_candid_url().to_owned();
        let candid = info.get_output_idl_path().to_owned();
        let build = info.get_build_tasks().to_owned();

        Ok(CustomBuilderExtra {
            dependencies,
            input_wasm_url,
            wasm,
            input_candid_url,
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
    #[context("Failed to create CustomBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(CustomBuilder {
            logger: env.get_logger().clone(),
        })
    }
}

impl CanisterBuilder for CustomBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(CustomBuilderExtra::try_from(info, pool)?.dependencies)
    }

    #[context("Failed to build custom canister {}.", info.get_name())]
    fn build(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let CustomBuilderExtra {
            input_candid_url: _,
            candid,
            input_wasm_url: _,
            wasm,
            build,
            dependencies,
        } = CustomBuilderExtra::try_from(info, pool)?;

        let canister_id = info.get_canister_id().unwrap();
        let vars = super::get_and_write_environment_variables(
            info,
            &config.network_name,
            pool,
            &dependencies,
            config.env_file.as_deref(),
        )?;

        for command in build {
            info!(
                self.logger,
                r#"{} '{}'"#,
                style("Executing").green().bold(),
                command
            );

            super::run_command(&command, &vars, info.get_workspace_root(), true)
                .with_context(|| format!("Failed to run {}.", command))?;
        }

        Ok(BuildOutput {
            canister_id,
            wasm: WasmBuildOutput::File(wasm),
            idl: IdlBuildOutput::File(candid),
        })
    }

    fn get_candid_path(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        // get the path to candid file
        let CustomBuilderExtra { candid, .. } = CustomBuilderExtra::try_from(info, pool)?;
        Ok(candid)
    }
}

pub async fn custom_download(info: &CanisterInfo, pool: &CanisterPool) -> DfxResult {
    let CustomBuilderExtra {
        input_candid_url,
        candid,
        input_wasm_url,
        wasm,
        build: _,
        dependencies: _,
    } = CustomBuilderExtra::try_from(info, pool)?;

    if let Some(url) = input_wasm_url {
        download_file_to_path(&url, &wasm).await?;
    }
    if let Some(url) = input_candid_url {
        download_file_to_path(&url, &candid).await?;
    }

    Ok(())
}
