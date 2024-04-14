use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::metadata::names::{CANDID_ARGS, CANDID_SERVICE};
use crate::lib::models::canister::CanisterPool;
use crate::lib::package_arguments::{self, PackageArguments};
use crate::util::assets::management_idl;
use crate::lib::builders::bail;
use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use dfx_core::config::cache::Cache;
use dfx_core::config::model::dfinity::{MetadataVisibility, Profile};
use dfx_core::fs::metadata;
use fn_error_context::context;
use slog::{info, o, trace, warn, Logger};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::Arc;

pub struct MotokoBuilder {
    logger: slog::Logger,
    cache: Arc<dyn Cache>,
}
unsafe impl Send for MotokoBuilder {}
unsafe impl Sync for MotokoBuilder {}

impl MotokoBuilder {
    #[context("Failed to create MotokoBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(MotokoBuilder {
            logger: env.get_logger().new(o! {
                "module" => "motoko"
            }),
            cache: env.get_cache(),
        })
    }
}

#[context("Failed to find imports for canister at '{}'.", info.get_main_path().display())]
fn get_imports(cache: &dyn Cache, info: &MotokoCanisterInfo) -> DfxResult<BTreeSet<MotokoImport>> {
    #[context("Failed recursive dependency detection at {}.", file.display())]
    fn get_imports_recursive(
        cache: &dyn Cache,
        file: &Path,
        result: &mut BTreeSet<MotokoImport>,
    ) -> DfxResult {
        if result.contains(&MotokoImport::Relative(file.to_path_buf())) {
            return Ok(());
        }

        result.insert(MotokoImport::Relative(file.to_path_buf()));

        let mut command = cache.get_binary_command("moc")?;
        let command = command.arg("--print-deps").arg(file);
        let output = command
            .output()
            .with_context(|| format!("Error executing {:#?}", command))?;
        let output = String::from_utf8_lossy(&output.stdout);

        for line in output.lines() {
            let import = MotokoImport::try_from(line).context("Failed to create MotokoImport.")?;
            match import {
                MotokoImport::Relative(path) => {
                    get_imports_recursive(cache, path.as_path(), result)?;
                }
                _ => {
                    result.insert(import);
                }
            }
        }

        Ok(())
    }

    let mut result = BTreeSet::new();
    get_imports_recursive(cache, info.get_main_path(), &mut result)?;

    Ok(result)
}

impl CanisterBuilder for MotokoBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;
        let imports = get_imports(self.cache.as_ref(), &motoko_info)?;

        Ok(imports
            .iter()
            .filter_map(|import| {
                if let MotokoImport::Canister(name) = import {
                    pool.get_first_canister_with_name(name)
                } else {
                    None
                }
            })
            .map(|canister| canister.canister_id())
            .collect())
    }

    #[context("Failed to build Motoko canister '{}'.", canister_info.get_name())]
    fn build(
        &self,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let motoko_info = canister_info.as_info::<MotokoCanisterInfo>()?;
        let profile = config.profile;
        let input_path = motoko_info.get_main_path();
        let output_wasm_path = motoko_info.get_output_wasm_path();

        // from name to principal:
        let id_map = pool
            .get_canister_list()
            .iter()
            .map(|c| (c.get_name().to_string(), c.canister_id().to_text()))
            .collect();
        // from principal to name:
        let rev_id_map: BTreeMap<String, String> = pool
            .get_canister_list()
            .iter()
            .map(|c| (c.canister_id().to_text(), c.get_name().to_string()))
            .collect();

        std::fs::create_dir_all(motoko_info.get_output_root()).with_context(|| {
            format!(
                "Failed to create {}.",
                motoko_info.get_output_root().to_string_lossy()
            )
        })?;
        let cache = &self.cache;
        let idl_dir_path = &config.idl_root;
        std::fs::create_dir_all(idl_dir_path)
            .with_context(|| format!("Failed to create {}.", idl_dir_path.to_string_lossy()))?;

        let imports = get_imports(cache.as_ref(), &motoko_info)?;

        // If the management canister is being imported, emit the candid file.
        if imports.contains(&MotokoImport::Ic("aaaaa-aa".to_string()))
        {
            let management_idl_path = idl_dir_path.join("aaaaa-aa.did");
            dfx_core::fs::write(management_idl_path, management_idl()?)?;
        }

        let package_arguments =
            package_arguments::load(cache.as_ref(), motoko_info.get_packtool())?;
        let mut package_arguments_map = BTreeMap::<String, String>::new(); // TODO: Can we deal without cloning strings?
        { // block
            let mut i = 0;
            while i + 3 <= package_arguments.len() {
                if package_arguments[i] == "--package" {
                    package_arguments_map.insert(package_arguments[i+1].clone(), package_arguments[i+2].clone());
                    i += 3;
                } else {
                    i += 1;
                }
            };
        }

        // Check that one of the dependencies is newer than the target:
        if let Ok(wasm_file_metadata) = metadata(output_wasm_path) {
            let wasm_file_time = wasm_file_metadata.modified()?;
            let mut import_iter = imports.iter();
            loop {
                if let Some(import) = import_iter.next() {
                    let imported_file = match import {
                        MotokoImport::Canister(canister_name) => {
                            if let Some(canister) = pool.get_first_canister_with_name(canister_name) {
                                let main_file = canister.get_info().get_main_file();
                                if let Some(main_file) = main_file {
                                    Some(main_file.to_owned())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        MotokoImport::Ic(canister_id) => {
                            if let Some(canister_name) = rev_id_map.get(canister_id) {
                                if let Some(canister) = pool.get_first_canister_with_name(canister_name) {
                                    if let Some(main_file) = canister.get_info().get_main_file() {
                                        Some(main_file.to_owned())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        MotokoImport::Lib(path) => {
                            let i = path.find('/');
                            let pre_path = if let Some(i) = i {
                                let expanded = Path::new(
                                    package_arguments_map.get(&path[..i]).ok_or_else(|| anyhow!("nonexisting package"))?
                                );
                                expanded.join(&path[i+1..])
                            } else {
                                Path::new(path).to_owned()
                            };
                            let path2 = pre_path.to_string_lossy() + ".mo"; // TODO: Is `lossy` OK?
                            let path2 = path2.to_string();
                            let path2 = Path::new(&path2);
                            if path2.exists() { // TODO: Is it correct order of two variants?
                                Some(Path::new(path2).to_owned())
                            } else {
                                let path3 = pre_path.join(Path::new("lib.mo"));
                                println!("path3: {}", &path3.to_string_lossy()); // FIXME
                                if path3.exists() {
                                    Some(path3.to_owned())
                                } else {
                                    bail!("source file has been deleted");
                                }
                            }
                        }
                        MotokoImport::Relative(path) => {
                            Some(Path::new(path).to_owned())
                        }
                    };
                    if let Some(imported_file) = imported_file {
                        let imported_file_metadata = metadata(imported_file.as_ref())?;
                        let imported_file_time = imported_file_metadata.modified()?;
                        if imported_file_time > wasm_file_time {
                            break;
                        };
                    };
                } else {
                    bail!("already compiled"); // FIXME: Ensure that `dfx` command doesn't return false because of this.
                }
            }
        };

        let moc_arguments = match motoko_info.get_args() {
            Some(args) => [
                package_arguments,
                args.split_whitespace().map(str::to_string).collect(),
            ]
            .concat(),
            None => package_arguments,
        };

        let candid_service_metadata_visibility = canister_info
            .get_metadata(CANDID_SERVICE)
            .map(|m| m.visibility)
            .unwrap_or(MetadataVisibility::Public);

        let candid_args_metadata_visibility = canister_info
            .get_metadata(CANDID_ARGS)
            .map(|m| m.visibility)
            .unwrap_or(MetadataVisibility::Public);

        // Generate wasm
        let params = MotokoParams {
            build_target: match profile {
                Profile::Release => BuildTarget::Release,
                _ => BuildTarget::Debug,
            },
            suppress_warning: false,
            input: input_path,
            package_arguments: &moc_arguments,
            candid_service_metadata_visibility,
            candid_args_metadata_visibility,
            output: output_wasm_path,
            idl_path: idl_dir_path,
            idl_map: &id_map,
        };
        motoko_compile(&self.logger, cache.as_ref(), &params)?;

        Ok(BuildOutput {
            canister_id: canister_info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(motoko_info.get_output_wasm_path().to_path_buf()),
            idl: IdlBuildOutput::File(motoko_info.get_output_idl_path().to_path_buf()),
        })
    }

    fn get_candid_path(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        // get the path to candid file from dfx build
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;
        let idl_from_build = motoko_info.get_output_idl_path().to_path_buf();
        Ok(idl_from_build)
    }
}

type CanisterIdMap = BTreeMap<String, String>;
enum BuildTarget {
    Release,
    Debug,
}

struct MotokoParams<'a> {
    build_target: BuildTarget,
    idl_path: &'a Path,
    idl_map: &'a CanisterIdMap,
    package_arguments: &'a PackageArguments,
    candid_service_metadata_visibility: MetadataVisibility,
    candid_args_metadata_visibility: MetadataVisibility,
    output: &'a Path,
    input: &'a Path,
    // The following fields are control flags for dfx and will not be used by self.to_args()
    suppress_warning: bool,
}

impl MotokoParams<'_> {
    fn to_args(&self, cmd: &mut std::process::Command) {
        cmd.arg(self.input);
        cmd.arg("-o").arg(self.output);
        match self.build_target {
            BuildTarget::Release => cmd.args(["-c", "--release"]),
            BuildTarget::Debug => cmd.args(["-c", "--debug"]),
        };
        cmd.arg("--idl").arg("--stable-types");
        if self.candid_service_metadata_visibility == MetadataVisibility::Public {
            // moc defaults to private metadata, if this argument is not present.
            cmd.arg("--public-metadata").arg(CANDID_SERVICE);
        }
        if self.candid_args_metadata_visibility == MetadataVisibility::Public {
            // moc defaults to private metadata, if this argument is not present.
            cmd.arg("--public-metadata").arg(CANDID_ARGS);
        }
        if !self.idl_map.is_empty() {
            cmd.arg("--actor-idl").arg(self.idl_path);
            for (name, canister_id) in self.idl_map.iter() {
                cmd.args(["--actor-alias", name, canister_id]);
            }
        };
        cmd.args(self.package_arguments);
    }
}

/// Compile a motoko file.
#[context("Failed to compile Motoko.")]
fn motoko_compile(logger: &Logger, cache: &dyn Cache, params: &MotokoParams<'_>) -> DfxResult {
    let mut cmd = cache.get_binary_command("moc")?;
    params.to_args(&mut cmd);
    run_command(logger, &mut cmd, params.suppress_warning).context("Failed to run 'moc'.")?;
    Ok(())
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum MotokoImport {
    Canister(String),
    Ic(String),
    Lib(String),
    Relative(PathBuf),
}

impl TryFrom<&str> for MotokoImport {
    type Error = DfxError;

    fn try_from(line: &str) -> Result<Self, DfxError> {
        let (url, fullpath) = match line.find(' ') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(DfxError::new(BuildError::DependencyError(format!(
                        "Unknown import {}",
                        line
                    ))));
                }
                let (url, fullpath) = line.split_at(index + 1);
                (url.trim_end(), Some(fullpath))
            }
            None => (line, None),
        };
        let import = match url.find(':') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(DfxError::new(BuildError::DependencyError(format!(
                        "Unknown import {}",
                        url
                    ))));
                }
                let (prefix, name) = url.split_at(index + 1);
                match prefix {
                    "canister:" => MotokoImport::Canister(name.to_owned()),
                    "ic:" => MotokoImport::Ic(name.to_owned()),
                    "mo:" => MotokoImport::Lib(name.to_owned()),
                    _ => {
                        return Err(DfxError::new(BuildError::DependencyError(format!(
                            "Unknown import {}",
                            url
                        ))))
                    }
                }
            }
            None => match fullpath {
                Some(fullpath) => {
                    let path = PathBuf::from(fullpath);
                    if !path.is_file() {
                        return Err(DfxError::new(BuildError::DependencyError(format!(
                            "Cannot find import file {}",
                            path.display()
                        ))));
                    };
                    MotokoImport::Relative(path)
                }
                None => {
                    return Err(DfxError::new(BuildError::DependencyError(format!(
                        "Cannot resolve relative import {}",
                        url
                    ))))
                }
            },
        };

        Ok(import)
    }
}

fn run_command(
    logger: &slog::Logger,
    cmd: &mut std::process::Command,
    suppress_warning: bool,
) -> DfxResult<Output> {
    trace!(logger, r#"Running {}..."#, format!("{:?}", cmd));

    let output = cmd.output().context("Error while executing command.")?;
    if !output.status.success() {
        Err(DfxError::new(BuildError::CommandError(
            format!("{:?}", cmd),
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    } else {
        if !output.stdout.is_empty() {
            info!(logger, "{}", String::from_utf8_lossy(&output.stdout));
        }
        if !suppress_warning && !output.stderr.is_empty() {
            warn!(logger, "{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(output)
    }
}
