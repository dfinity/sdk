use crate::config::cache::Cache;
use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util;

use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use fn_error_context::context;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Set of extras that can be specified in the dfx.json.
struct AssetsBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
}

impl AssetsBuilderExtra {
    #[context("Failed to create AssetBuilderExtra for canister '{}'.", info.get_name())]
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

        Ok(AssetsBuilderExtra { dependencies })
    }
}
pub struct AssetsBuilder {
    _cache: Arc<dyn Cache>,
}

impl AssetsBuilder {
    #[context("Failed to create AssetBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(AssetsBuilder {
            _cache: env.get_cache(),
        })
    }
}

impl CanisterBuilder for AssetsBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(AssetsBuilderExtra::try_from(info, pool)?.dependencies)
    }

    #[context("Failed to build asset canister '{}'.", info.get_name())]
    fn build(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let mut canister_assets = util::assets::assetstorage_canister()
            .context("Failed to get asset canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read asset canister archive entries.")?
        {
            let mut file = file.context("Failed to read asset canister archive entry.")?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            // See https://github.com/alexcrichton/tar-rs/issues/261
            fs::create_dir_all(info.get_output_root()).with_context(|| {
                format!(
                    "Failed to create {}.",
                    info.get_output_root().to_string_lossy()
                )
            })?;
            file.unpack_in(info.get_output_root()).with_context(|| {
                format!(
                    "Failed to unpack archive to {}.",
                    info.get_output_root().to_string_lossy()
                )
            })?;
        }

        let wasm_path = info
            .get_output_root()
            .join(Path::new("assetstorage.wasm.gz"));
        let idl_path = info.get_output_root().join(Path::new("assetstorage.did"));
        Ok(BuildOutput {
            canister_id: info.get_canister_id().expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(wasm_path),
            idl: IdlBuildOutput::File(idl_path),
            add_candid_service_metadata: false,
        })
    }

    fn postbuild(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult {
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

        let vars = super::environment_variables(info, &config.network_name, pool, &dependencies);

        build_frontend(
            pool.get_logger(),
            info.get_workspace_root(),
            &config.network_name,
            vars,
        )?;

        let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
        assets_canister_info.assert_source_paths()?;

        Ok(())
    }

    #[context("Failed to generate idl for canister '{}'.", info.get_name())]
    fn generate_idl(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<std::path::PathBuf> {
        let generate_output_dir = info
            .get_declarations_config()
            .output
            .as_ref()
            .context("`declarations.output` must not be None")?;

        let mut canister_assets = util::assets::assetstorage_canister()
            .context("Failed to load asset canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read asset canister archive entries.")?
        {
            let mut file = file.context("Failed to read asset canister archive entry.")?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            // See https://github.com/alexcrichton/tar-rs/issues/261
            fs::create_dir_all(&generate_output_dir).with_context(|| {
                format!(
                    "Failed to create {}.",
                    generate_output_dir.to_string_lossy()
                )
            })?;

            file.unpack_in(generate_output_dir.clone())
                .with_context(|| {
                    format!(
                        "Failed to unpack archive content to {}.",
                        generate_output_dir.to_string_lossy()
                    )
                })?;
        }

        // delete unpacked wasm file
        let wasm_path = generate_output_dir.join(Path::new("assetstorage.wasm.gz"));
        if wasm_path.exists() {
            std::fs::remove_file(&wasm_path)
                .with_context(|| format!("Failed to remove {}.", wasm_path.to_string_lossy()))?;
        }

        let idl_path = generate_output_dir.join(Path::new("assetstorage.did"));
        let idl_path_rename = generate_output_dir
            .join(info.get_name())
            .with_extension("")
            .with_extension("did");
        if idl_path.exists() {
            std::fs::rename(&idl_path, &idl_path_rename)
                .with_context(|| format!("Failed to rename {}.", idl_path.to_string_lossy()))?;
        }

        Ok(idl_path_rename)
    }
}

#[context("Failed to build frontend for network '{}'.", network_name)]
fn build_frontend(
    logger: &slog::Logger,
    project_root: &Path,
    network_name: &str,
    vars: Vec<super::Env<'_>>,
) -> DfxResult {
    let build_frontend = project_root.join("package.json").exists();
    // If there is not a package.json, we don't have a frontend and can quit early.

    if build_frontend {
        // Frontend build.
        slog::info!(logger, "Building frontend...");
        let mut cmd = std::process::Command::new("npm");

        // Provide DFX_NETWORK at build time
        cmd.env("DFX_NETWORK", network_name);

        cmd.arg("run").arg("build");

        if NetworkDescriptor::is_ic(network_name, &vec![]) {
            cmd.env("NODE_ENV", "production");
        }

        for (var, value) in vars {
            cmd.env(var.as_ref(), value);
        }

        cmd.current_dir(project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        slog::debug!(logger, "Running {:?}...", cmd);

        let output = cmd
            .output()
            .with_context(|| format!("Error executing {:#?}", cmd))?;
        if !output.status.success() {
            return Err(DfxError::new(BuildError::CommandError(
                format!("{:?}", cmd),
                output.status,
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )));
        } else if !output.stderr.is_empty() {
            // Cannot use eprintln, because it would interfere with the progress bar.
            slog::warn!(logger, "{}", String::from_utf8_lossy(&output.stderr));
        }
    }
    Ok(())
}
