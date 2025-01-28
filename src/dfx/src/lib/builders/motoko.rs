use itertools::Itertools;
use crate::config::cache::VersionCache;
use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::metadata::names::{CANDID_ARGS, CANDID_SERVICE};
use crate::lib::models::canister::{CanisterPool, Import};
use crate::lib::package_arguments::{self, PackageArguments};
use crate::util::assets::management_idl;
use anyhow::Context;
use candid::Principal as CanisterId;
use dfx_core::config::model::dfinity::{MetadataVisibility, Profile};
use fn_error_context::context;
use slog::{info, o, trace, warn, Logger};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Output;

pub struct MotokoBuilder {
    logger: slog::Logger,
    cache: VersionCache,
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

// #[context("Failed to find imports for canister at '{}'.", info.get_main_path().display())]
// fn get_imports(
//     env: &dyn Environment,
//     cache: &VersionCache,
//     info: &MotokoCanisterInfo,
// ) -> DfxResult<BTreeSet<Import>> {
//     #[context("Failed recursive dependency detection at {}.", file.display())]
//     fn get_imports_recursive(
//         env: &dyn Environment,
//         cache: &VersionCache,
//         workspace_root: &Path,
//         file: &Path,
//         result: &mut BTreeSet<Import>,
//     ) -> DfxResult {
//         if result.contains(&MotokoImport::Relative(file.to_path_buf())) {
//             return Ok(());
//         }

//         result.insert(MotokoImport::Relative(file.to_path_buf()));

//         let mut command = cache.get_binary_command(env, "moc")?;
//         command.current_dir(workspace_root);
//         let command = command.arg("--print-deps").arg(file);
//         let output = command
//             .output()
//             .with_context(|| format!("Error executing {:#?}", command))?;
//         let output = String::from_utf8_lossy(&output.stdout);

//         for line in output.lines() {
//             let import = MotokoImport::try_from(line).context("Failed to create MotokoImport.")?;
//             match import {
//                 MotokoImport::Relative(path) => {
//                     get_imports_recursive(env, cache, workspace_root, path.as_path(), result)?;
//                 }
//                 _ => {
//                     result.insert(import);
//                 }
//             }
//         }

//         Ok(())
//     }

//     let mut result = BTreeSet::new();
//     get_imports_recursive(
//         env,
//         cache,
//         info.get_workspace_root(),
//         info.get_main_path(),
//         &mut result,
//     )?;

//     Ok(result)
// }

impl CanisterBuilder for MotokoBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        env: &dyn Environment,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        self.read_dependencies(env, pool, info)?;

        let imports = env.get_imports().borrow();
        let graph = imports.graph();
        // let space = DfsSpace::new(&graph);
        // match petgraph::algo::toposort(graph, Some(&mut space)) { // FIXME: Should provide the node.
        // TODO: inefficient:
        match petgraph::algo::toposort(graph, None) {
            Ok(order) => {
                let res: Vec<_> = order
                    .into_iter()
                    .filter_map(|id| match graph.node_weight(id) {
                        Some(Import::Canister(name)) => {
                            pool.get_first_canister_with_name(name.as_str()) // TODO: a little inefficient
                        }
                        _ => None,
                    })
                    .map(|canister| canister.canister_id())
                    .collect();
                let main_canister_id = info.get_canister_id()?;
                if let Some(start_index) = res.iter().position(|&x| x == main_canister_id) {
                    // Create a slice starting from that index
                    let slice = &res[start_index..]; // TODO: Include or not the canister itself?
                    Ok(slice.to_vec())
                } else {
                    panic!("Programming error");
                }
            }
            Err(err) => {
                let message = match graph.node_weight(err.node_id()) {
                    Some(Import::Canister(name)) => name.clone(),
                    _ => "<Unknown>".to_string(),
                };
                return Err(DfxError::new(BuildError::DependencyError(format!(
                    "Found circular dependency: {}",
                    message
                ))));
            }
        }
    }

    #[context("Failed to build Motoko canister '{}'.", canister_info.get_name())]
    fn build(
        &self,
        env: &dyn Environment,
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
        std::fs::create_dir_all(idl_dir_path)
            .with_context(|| format!("Failed to create {}.", idl_dir_path.to_string_lossy()))?;

        // If the management canister is being imported, emit the candid file.
        if env.get_imports().borrow().nodes().keys().contains(&Import::Canister("aaaaa-aa".to_string()))
        {
            let management_idl_path = idl_dir_path.join("aaaaa-aa.did");
            dfx_core::fs::write(management_idl_path, management_idl()?)?;
        }

        let dependencies = self
            .get_dependencies(env, pool, canister_info)
            .unwrap_or_default();
        super::get_and_write_environment_variables(
            canister_info,
            &config.network_name,
            pool,
            &dependencies,
            config.env_file.as_deref(),
        )?;

        let package_arguments = package_arguments::load(
            env,
            cache,
            motoko_info.get_packtool(),
            canister_info.get_workspace_root(),
        )?;

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
            workspace_root: canister_info.get_workspace_root(),
        };
        motoko_compile(env, &self.logger, cache, &params)?;

        Ok(BuildOutput {
            canister_id: canister_info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(motoko_info.get_output_wasm_path().to_path_buf()),
            idl: IdlBuildOutput::File(canister_info.get_output_idl_path().to_path_buf()),
        })
    }

    fn get_candid_path(
        &self,
        _: &dyn Environment,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        // get the path to candid file from dfx build
        Ok(info.get_output_idl_path().to_path_buf())
    }
}

type CanisterIdMap = BTreeMap<String, String>;
enum BuildTarget {
    Release,
    Debug,
}

struct MotokoParams<'a> {
    build_target: BuildTarget,
    workspace_root: &'a Path,
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
fn motoko_compile(
    env: &dyn Environment,
    logger: &Logger,
    cache: &VersionCache,
    params: &MotokoParams<'_>,
) -> DfxResult {
    let mut cmd = cache.get_binary_command(env, "moc")?;
    cmd.current_dir(params.workspace_root);
    params.to_args(&mut cmd);
    run_command(logger, &mut cmd, params.suppress_warning).context("Failed to run 'moc'.")?;
    Ok(())
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
