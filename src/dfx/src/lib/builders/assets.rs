use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::lib::program;
use crate::util;
use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use console::style;
use dfx_core::config::cache::Cache;
use fn_error_context::context;
use slog::{o, Logger};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Set of extras that can be specified in the dfx.json.
struct AssetsBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
    /// A command to run to build this canister's assets. This is optional if
    /// the canister does not have a frontend or can be built using the default
    /// `npm run build` command.
    build: Vec<String>,
    /// The NPM workspace that the project is located in.
    workspace: Option<String>,
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
        let info = info.as_info::<AssetsCanisterInfo>()?;
        let build = info.get_build_tasks().to_owned();
        let workspace = info.get_npm_workspace().map(str::to_owned);

        Ok(AssetsBuilderExtra {
            dependencies,
            build,
            workspace,
        })
    }
}
pub struct AssetsBuilder {
    _cache: Arc<dyn Cache>,
    logger: Logger,
}
unsafe impl Send for AssetsBuilder {}
unsafe impl Sync for AssetsBuilder {}

impl AssetsBuilder {
    #[context("Failed to create AssetBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(AssetsBuilder {
            _cache: env.get_cache(),
            logger: env.get_logger().new(o!("module" => "assets")),
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
        let wasm_path = info
            .get_output_root()
            .join(Path::new("assetstorage.wasm.gz"));
        unpack_did(info.get_output_root())?;
        let canister_assets = util::assets::assets_wasm(&self.logger)?;
        fs::write(&wasm_path, canister_assets).context("Failed to write asset canister wasm")?;
        let idl_path = info.get_output_root().join(Path::new("assetstorage.did"));
        Ok(BuildOutput {
            canister_id: info.get_canister_id().expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(wasm_path),
            idl: IdlBuildOutput::File(idl_path),
        })
    }

    fn postbuild(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult {
        let AssetsBuilderExtra {
            build,
            dependencies,
            workspace,
        } = AssetsBuilderExtra::try_from(info, pool)?;

        let vars = super::get_and_write_environment_variables(
            info,
            &config.network_name,
            pool,
            &dependencies,
            config.env_file.as_deref(),
        )?;

        build_frontend(
            pool.get_logger(),
            info.get_workspace_root(),
            &config.network_name,
            vars,
            &build,
            workspace.as_deref(),
        )?;

        let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
        assets_canister_info.assert_source_paths()?;

        Ok(())
    }

    fn get_candid_path(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<std::path::PathBuf> {
        let idl_path = info.get_output_root().join(Path::new("assetstorage.did"));
        Ok(idl_path)
    }
}

fn unpack_did(generate_output_dir: &Path) -> DfxResult<()> {
    let mut canister_assets =
        util::assets::assetstorage_canister().context("Failed to load asset canister archive.")?;
    for file in canister_assets
        .entries()
        .context("Failed to read asset canister archive entries.")?
    {
        let mut file = file.context("Failed to read asset canister archive entry.")?;

        if !file.header().entry_type().is_dir() && file.path()?.ends_with("assetstorage.did") {
            // See https://github.com/alexcrichton/tar-rs/issues/261
            fs::create_dir_all(generate_output_dir)
                .with_context(|| format!("Failed to create {}.", generate_output_dir.display()))?;
            file.unpack_in(generate_output_dir).with_context(|| {
                format!(
                    "Failed to unpack archive content to {}.",
                    generate_output_dir.display()
                )
            })?;
            break;
        }
    }
    Ok(())
}

#[context("Failed to build frontend for network '{}'.", network_name)]
fn build_frontend(
    logger: &slog::Logger,
    project_root: &Path,
    network_name: &str,
    vars: Vec<super::Env<'_>>,
    build: &[String],
    workspace: Option<&str>,
) -> DfxResult {
    let custom_build_frontend = !build.is_empty();
    let build_frontend = project_root.join("package.json").exists();
    // If there is no package.json or custom build command, we don't have a frontend and can quit early.

    if custom_build_frontend {
        for command in build {
            slog::info!(
                logger,
                r#"{} '{}'"#,
                style("Executing").green().bold(),
                command
            );

            super::run_command(command, &vars, project_root)
                .with_context(|| format!("Failed to run {}.", command))?;
        }
    } else if build_frontend {
        // Frontend build.
        slog::info!(logger, "Building frontend...");
        let mut cmd = std::process::Command::new(program::NPM);

        // Provide DFX_NETWORK at build time
        cmd.env("DFX_NETWORK", network_name);

        cmd.arg("run").arg("build");

        if let Some(workspace) = workspace {
            cmd.arg("--workspace").arg(workspace);
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
            slog::debug!(
                logger,
                "Frontend build succeed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    Ok(())
}
