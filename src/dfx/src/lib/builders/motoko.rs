use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::metadata::names::{CANDID_ARGS, CANDID_SERVICE};
use crate::lib::models::canister::{CanisterPool, Import, ImportsTracker};
use crate::lib::package_arguments::{self, PackageArguments};
use crate::util::assets::management_idl;
use anyhow::Context;
use candid::Principal as CanisterId;
use dfx_core::config::cache::Cache;
use dfx_core::config::model::dfinity::{MetadataVisibility, Profile};
use fn_error_context::context;
use slog::{info, o, trace, warn, Logger};
use std::collections::BTreeMap;
use std::convert::TryFrom;
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

/// Add imports originating from canister `info` to the graph `imports` of dependencies.
#[context("Failed to find imports for canister at '{}'.", info.as_info::<MotokoCanisterInfo>().unwrap().get_main_path().display())]
pub fn add_imports(
    cache: &dyn Cache,
    info: &CanisterInfo,
    imports: &mut ImportsTracker,
    pool: &CanisterPool,
) -> DfxResult<()> {
    let motoko_info = info.as_info::<MotokoCanisterInfo>()?;
    #[context("Failed recursive dependency detection at {}.", file.display())]
    fn add_imports_recursive(
        cache: &dyn Cache,
        file: &Path,
        imports: &mut ImportsTracker,
        pool: &CanisterPool,
        top: Option<&CanisterInfo>, // hackish
    ) -> DfxResult {
        let base_path = file.parent().unwrap(); // FIXME: `unwrap()`
        let parent = if let Some(top) = top {
            Import::Canister(top.get_name().to_string()) // a little inefficient
        } else {
            Import::FullPath(base_path.join(file))
        };
        if imports.nodes.get(&parent).is_some() {
            // The item is already in the graph.
            return Ok(());
        } else {
            imports
                .nodes
                .insert(parent.clone(), imports.graph.add_node(parent.clone()));
        }

        let mut command = cache.get_binary_command("moc")?;
        let command = command.arg("--print-deps").arg(file);
        let output = command
            .output()
            .with_context(|| format!("Error executing {:#?}", command))?;
        let output = String::from_utf8_lossy(&output.stdout);

        for line in output.lines() {
            let child = Import::try_from(line).context("Failed to create MotokoImport.")?;
            match &child {
                Import::FullPath(full_child_path) => {
                    // duplicate code
                    let path2 = full_child_path.join(Path::new("lib.mo"));
                    let child_path = if path2.exists() {
                        &path2
                    } else {
                        full_child_path
                    };
                    add_imports_recursive(cache, child_path.as_path(), imports, pool, None)?;
                }
                Import::Canister(canister_name) => {
                    // duplicate code
                    if let Some(canister) =
                        pool.get_first_canister_with_name(canister_name.as_str())
                    {
                        let main_file = canister.get_info().get_main_file();
                        if let Some(main_file) = main_file {
                            add_imports_recursive(
                                cache,
                                Path::new(main_file),
                                imports,
                                pool,
                                Some(canister.get_info()),
                            )?;
                        }
                    }
                }
                _ => {}
            }
            let parent_node_index = *imports
                .nodes
                .entry(parent.clone())
                .or_insert_with(|| imports.graph.add_node(parent.clone()));
            let child_node_index = *imports
                .nodes
                .entry(child.clone())
                .or_insert_with(|| imports.graph.add_node(child.clone()));
            imports
                .graph
                .update_edge(parent_node_index, child_node_index, ());
        }

        Ok(())
    }

    add_imports_recursive(
        cache,
        motoko_info.get_main_path().canonicalize()?.as_path(),
        imports,
        pool,
        Some(info),
    )?;

    Ok(())
}

impl CanisterBuilder for MotokoBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        add_imports(
            self.cache.as_ref(),
            info,
            &mut pool.imports.borrow_mut(),
            pool,
        )?;

        let graph = &pool.imports.borrow().graph;
        match petgraph::algo::toposort(&pool.imports.borrow().graph, None) {
            Ok(order) => {
                Ok(order
                    .into_iter()
                    .filter_map(|id| match graph.node_weight(id) {
                        Some(Import::Canister(name)) => {
                            pool.get_first_canister_with_name(name.as_str()) // TODO: a little inefficient
                        }
                        _ => None,
                    })
                    .map(|canister| canister.canister_id())
                    .collect())
            }
            Err(err) => {
                let message = match graph.node_weight(err.node_id()) {
                    Some(Import::Canister(name)) => name,
                    _ => "<Unknown>",
                };
                return Err(DfxError::new(BuildError::DependencyError(format!(
                    "Found circular dependency: {}",
                    message
                ))));
            }
        }
    }

    /// TODO: Ideally, should make inter-canister dependencies to rely on `.did` file changed or not.
    #[context("Failed to build Motoko canister '{}'.", canister_info.get_name())]
    fn build(
        &self,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let motoko_info = canister_info.as_info::<MotokoCanisterInfo>()?; // TODO: Remove.
        let profile = config.profile;
        let input_path = motoko_info.get_main_path();
        let output_wasm_path = canister_info.get_output_wasm_path();

        // from name to principal:
        let id_map = pool
            .get_canister_list()
            .iter()
            .filter(|&c| canister_info.get_dependencies().iter().map(|s| s.as_str()).find(|&name| name == c.get_name()).is_some()) // TODO: 1. Slow. 2. Use Motoko dependencies where appropriate.
            .map(|c| (c.get_name().to_string(), c.canister_id().to_text()))
            .collect();

        std::fs::create_dir_all(motoko_info.get_output_root()).with_context(|| {
            format!(
                "Failed to create {}.",
                motoko_info.get_output_root().to_string_lossy()
            )
        })?;
        let idl_dir_path = &config.idl_root;
        std::fs::create_dir_all(idl_dir_path)
            .with_context(|| format!("Failed to create {}.", idl_dir_path.to_string_lossy()))?;

        // If the management canister is being imported, emit the candid file.
        if pool
            .imports
            .borrow()
            .nodes
            .contains_key(&Import::Ic("aaaaa-aa".to_string()))
        {
            let management_idl_path = idl_dir_path.join("aaaaa-aa.did");
            dfx_core::fs::write(management_idl_path, management_idl()?)?;
        }

        let cache = &self.cache;

        let package_arguments =
            package_arguments::load(cache.as_ref(), motoko_info.get_packtool())?;
        let mut package_arguments_map = BTreeMap::<&str, &str>::new();
        {
            // block
            let mut i = 0;
            while i + 3 <= package_arguments.len() {
                if package_arguments[i] == "--package" {
                    package_arguments_map
                        .insert(&package_arguments[i + 1], &package_arguments[i + 2]);
                    i += 3;
                } else {
                    i += 1;
                }
            }
        }

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
            // duplicate code
            canister_id: canister_info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(canister_info.get_output_wasm_path().to_path_buf()),
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

impl TryFrom<&str> for Import {
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
                    "canister:" => Import::Canister(name.to_owned()),
                    "ic:" => Import::Ic(name.to_owned()),
                    "mo:" => Import::Lib(name.to_owned()),
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
                    if !path.is_file() { // FIXME: What's about `/lib.mo` paths?
                        return Err(DfxError::new(BuildError::DependencyError(format!(
                            "Cannot find import file {}",
                            path.display()
                        ))));
                    };
                    Import::FullPath(path) // TODO: `""` is a hack.
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
