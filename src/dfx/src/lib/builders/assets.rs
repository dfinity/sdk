use crate::config::cache::Cache;
use crate::config::dfx_version;
use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::util;
use ic_types::principal::Principal as CanisterId;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

/// Set of extras that can be specified in the dfx.json.
struct AssetsBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
}

impl AssetsBuilderExtra {
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

        Ok(AssetsBuilderExtra { dependencies })
    }
}
pub struct AssetsBuilder {
    _cache: Arc<dyn Cache>,
}

impl AssetsBuilder {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(AssetsBuilder {
            _cache: env.get_cache(),
        })
    }
}

impl CanisterBuilder for AssetsBuilder {
    fn supports(&self, info: &CanisterInfo) -> bool {
        info.get_type() == "assets"
    }

    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(AssetsBuilderExtra::try_from(info, pool)?.dependencies)
    }

    fn build(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let mut canister_assets = util::assets::assetstorage_canister()?;
        for file in canister_assets.entries()? {
            let mut file = file?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            file.unpack_in(info.get_output_root())?;
        }

        let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
        delete_output_directory(&info, &assets_canister_info)?;

        let wasm_path = info.get_output_root().join(Path::new("assetstorage.wasm"));
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

        build_frontend(
            pool.get_logger(),
            info.get_workspace_root(),
            &config.network_name,
            dependencies,
            pool,
        )?;

        let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
        assets_canister_info.assert_source_paths()?;

        copy_assets(pool.get_logger(), &assets_canister_info)?;
        Ok(())
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn delete_output_directory(
    info: &CanisterInfo,
    assets_canister_info: &AssetsCanisterInfo,
) -> DfxResult {
    let output_assets_path = assets_canister_info.get_output_assets_path();
    if output_assets_path.exists() {
        let output_assets_path = output_assets_path.canonicalize()?;
        if !output_assets_path.starts_with(info.get_workspace_root()) {
            return Err(DfxError::DirectoryIsOutsideWorkspaceRoot(
                output_assets_path,
            ));
        }
        fs::remove_dir_all(output_assets_path)?;
    }
    Ok(())
}

fn copy_assets(logger: &slog::Logger, assets_canister_info: &AssetsCanisterInfo) -> DfxResult {
    let source_paths = assets_canister_info.get_source_paths();
    let output_assets_path = assets_canister_info.get_output_assets_path();

    for source_path in source_paths {
        // If the source doesn't exist, we ignore it.
        if !source_path.exists() {
            slog::warn!(
                logger,
                r#"Source path "{}" does not exist."#,
                source_path.to_string_lossy()
            );

            continue;
        }

        let input_assets_path = source_path.as_path();
        let walker = WalkDir::new(input_assets_path).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry?;
            let source = entry.path();
            let relative = source
                .strip_prefix(input_assets_path)
                .expect("cannot strip prefix");

            let destination = output_assets_path.join(relative);

            // If the destination exists, we simply continue. We delete the output directory
            // prior to building so the only way this exists is if it's an output to one
            // of the build steps.
            if destination.exists() {
                continue;
            }

            if entry.file_type().is_dir() {
                fs::create_dir(destination)?;
            } else {
                fs::copy(source, destination)?;
            }
        }
    }
    Ok(())
}

fn build_frontend(
    logger: &slog::Logger,
    project_root: &Path,
    network_name: &str,
    dependencies: Vec<CanisterId>,
    pool: &CanisterPool,
) -> DfxResult {
    let build_frontend = project_root.join("package.json").exists();
    // If there is not a package.json, we don't have a frontend and can quit early.

    if build_frontend {
        // Frontend build.
        slog::info!(logger, "Building frontend...");
        let mut cmd = std::process::Command::new("npm");
        cmd.arg("run")
            .arg("build")
            .env("DFX_VERSION", &format!("{}", dfx_version()))
            .env("DFX_NETWORK", &network_name);

        for deps in &dependencies {
            let canister = pool.get_canister(deps).unwrap();
            if let Some(output) = canister.get_build_output() {
                let candid_path = match &output.idl {
                    IdlBuildOutput::File(p) => p.as_os_str(),
                };

                cmd.env(
                    format!("CANISTER_CANDID_PATH_{}", canister.get_name()),
                    candid_path,
                );
            }
        }

        cmd.current_dir(project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        slog::debug!(logger, "Running {:?}...", cmd);

        let output = cmd.output()?;
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
