use crate::config::cache::Cache;
use crate::config::dfinity::Profile;
use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::metadata::names::CANDID_SERVICE;
use crate::lib::models::canister::CanisterPool;
use crate::lib::package_arguments::{self, PackageArguments};

use anyhow::Context;
use candid::Principal as CanisterId;
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

impl CanisterBuilder for MotokoBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        let mut result = BTreeSet::new();
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;

        #[context("Failed recursive dependency detection at {}.", file.to_string_lossy())]
        fn find_deps_recursive(
            cache: &dyn Cache,
            file: &Path,
            result: &mut BTreeSet<MotokoImport>,
        ) -> DfxResult {
            if result.contains(&MotokoImport::Relative(file.to_path_buf())) {
                return Ok(());
            }

            let mut command = cache.get_binary_command("moc")?;
            let command = command.arg("--print-deps").arg(&file);
            let output = command
                .output()
                .with_context(|| format!("Error executing {:#?}", command))?;

            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                let import =
                    MotokoImport::try_from(line).context("Failed to create MotokoImport.")?;
                match import {
                    MotokoImport::Canister(_) => {
                        result.insert(import);
                    }
                    MotokoImport::Relative(path) => {
                        find_deps_recursive(cache, path.as_path(), result)?;
                    }
                    MotokoImport::Lib(_) => (),
                    MotokoImport::Ic(_) => (),
                }
            }

            Ok(())
        }
        find_deps_recursive(
            self.cache.as_ref(),
            motoko_info.get_main_path(),
            &mut result,
        )?;

        Ok(result
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

        let id_map = pool
            .get_canister_list()
            .iter()
            .map(|c| (c.get_name().to_string(), c.canister_id().to_text()))
            .collect();

        std::fs::create_dir_all(motoko_info.get_output_root()).with_context(|| {
            format!(
                "Failed to create {}.",
                motoko_info.get_output_root().to_string_lossy()
            )
        })?;
        let cache = &self.cache;
        let idl_dir_path = &config.idl_root;
        std::fs::create_dir_all(&idl_dir_path)
            .with_context(|| format!("Failed to create {}.", idl_dir_path.to_string_lossy()))?;

        let package_arguments =
            package_arguments::load(cache.as_ref(), motoko_info.get_packtool())?;

        let moc_arguments = match motoko_info.get_args() {
            Some(args) => [
                package_arguments,
                args.split_whitespace().map(str::to_string).collect(),
            ]
            .concat(),
            None => package_arguments,
        };

        // Generate wasm
        let params = MotokoParams {
            build_target: match profile {
                Profile::Release => BuildTarget::Release,
                _ => BuildTarget::Debug,
            },
            suppress_warning: false,
            input: input_path,
            package_arguments: &moc_arguments,
            output: output_wasm_path,
            idl_path: idl_dir_path,
            idl_map: &id_map,
        };
        motoko_compile(&self.logger, cache.as_ref(), &params)?;

        if canister_info.get_shrink() {
            info!(self.logger, "Shrink WASM module size.");
            super::shrink_wasm(motoko_info.get_output_wasm_path())?;
        }
        Ok(BuildOutput {
            canister_id: canister_info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(motoko_info.get_output_wasm_path().to_path_buf()),
            idl: IdlBuildOutput::File(motoko_info.get_output_idl_path().to_path_buf()),
            add_candid_service_metadata: false,
        })
    }

    fn generate_idl(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        let generate_output_dir = &info
            .get_declarations_config()
            .output
            .as_ref()
            .context("output here must not be None")?;

        std::fs::create_dir_all(generate_output_dir).with_context(|| {
            format!(
                "Failed to create {}.",
                generate_output_dir.to_string_lossy()
            )
        })?;

        let output_idl_path = generate_output_dir
            .join(info.get_name())
            .with_extension("did");

        // get the path to candid file from dfx build
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;
        let idl_from_build = motoko_info.get_output_idl_path().to_path_buf();

        std::fs::copy(&idl_from_build, &output_idl_path).with_context(|| {
            format!(
                "Failed to copy {} to {}.",
                idl_from_build.to_string_lossy(),
                output_idl_path.to_string_lossy()
            )
        })?;

        Ok(output_idl_path)
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
            BuildTarget::Release => cmd.args(&["-c", "--release"]),
            BuildTarget::Debug => cmd.args(&["-c", "--debug"]),
        };
        cmd.arg("--idl").arg("--stable-types");
        // TODO add a flag in dfx.json to opt-out public interface
        cmd.arg("--public-metadata").arg(CANDID_SERVICE);
        if !self.idl_map.is_empty() {
            cmd.arg("--actor-idl").arg(self.idl_path);
            for (name, canister_id) in self.idl_map.iter() {
                cmd.args(&["--actor-alias", name, canister_id]);
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
