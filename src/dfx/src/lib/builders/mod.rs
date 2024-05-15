use crate::config::dfx_version_str;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister::Import;
use anyhow::{bail, Context};
use candid::Principal as CanisterId;
use candid_parser::utils::CandidSource;
use dfx_core::config::cache::Cache;
use dfx_core::config::model::dfinity::{Config, Profile};
use dfx_core::network::provider::get_network_context;
use dfx_core::util;
use fn_error_context::context;
use handlebars::Handlebars;
use petgraph::visit::Bfs;
use slog::trace;
use slog::Logger;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{self, metadata};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

mod assets;
mod custom;
mod motoko;
mod pull;
mod rust;

pub use custom::custom_download;

use super::canister_info::motoko::MotokoCanisterInfo;

#[derive(Debug)]
pub enum WasmBuildOutput {
    // Wasm(Vec<u8>),
    File(PathBuf),
    // pull dependencies has no wasm output to be installed by `dfx canister install` or `dfx deploy`
    None,
}

#[derive(Debug)]
pub enum IdlBuildOutput {
    // IDLProg(IDLProg),
    File(PathBuf),
}

/// The output of a build.
#[derive(Debug)]
pub struct BuildOutput {
    pub canister_id: CanisterId,
    pub wasm: WasmBuildOutput,
    pub idl: IdlBuildOutput,
}

/// A stateless canister builder. This is meant to not keep any state and be passed everything.
pub trait CanisterBuilder {
    /// Returns the dependencies of this canister, if any. This should not be a transitive
    /// list.
    fn get_dependencies(
        &self,
        _env: &dyn Environment,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(Vec::new())
    }

    fn prebuild(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult {
        Ok(())
    }

    /// Build a canister. The canister contains all information related to a single canister,
    /// while the config contains information related to this particular build.
    fn build(
        &self,
        env: &dyn Environment,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput>;

    fn postbuild(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult {
        Ok(())
    }

    /// Generate type declarations for the canister
    fn generate(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult {
        let generate_output_dir = info
            .get_declarations_config()
            .output
            .as_ref()
            .context("`output` must not be None")?;

        if generate_output_dir.exists() {
            let generate_output_dir = dfx_core::fs::canonicalize(generate_output_dir)
                .with_context(|| {
                    format!(
                        "Failed to canonicalize output dir {}.",
                        generate_output_dir.to_string_lossy()
                    )
                })?;
            if !generate_output_dir.starts_with(info.get_workspace_root()) {
                bail!(
                    "Directory at '{}' is outside the workspace root.",
                    generate_output_dir.as_path().display()
                );
            }
            std::fs::remove_dir_all(&generate_output_dir).with_context(|| {
                format!(
                    "Failed to remove dir: {}",
                    generate_output_dir.to_string_lossy()
                )
            })?;
        }

        let bindings = info
            .get_declarations_config()
            .bindings
            .as_ref()
            .context("`bindings` must not be None")?;

        if bindings.is_empty() {
            eprintln!("`{}.declarations.bindings` in dfx.json was set to be an empty list, so no type declarations will be generated.", &info.get_name());
            return Ok(());
        }

        eprintln!(
            "Generating type declarations for canister {}:",
            &info.get_name()
        );

        std::fs::create_dir_all(generate_output_dir).with_context(|| {
            format!(
                "Failed to create dir: {}",
                generate_output_dir.to_string_lossy()
            )
        })?;

        let did_from_build = self.get_candid_path(pool, info, config)?;
        if !did_from_build.exists() {
            bail!(
                "Candid file: {} doesn't exist.",
                did_from_build.to_string_lossy()
            );
        }

        let (env, ty) = CandidSource::File(did_from_build.as_path()).load()?;

        // Typescript
        if bindings.contains(&"ts".to_string()) {
            let output_did_ts_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did.d.ts");
            let content =
                ensure_trailing_newline(candid_parser::bindings::typescript::compile(&env, &ty));
            std::fs::write(&output_did_ts_path, content).with_context(|| {
                format!(
                    "Failed to write to {}.",
                    output_did_ts_path.to_string_lossy()
                )
            })?;
            eprintln!("  {}", &output_did_ts_path.display());

            compile_handlebars_files("ts", info, generate_output_dir)?;
        }

        // Javascript
        if bindings.contains(&"js".to_string()) {
            // <canister.did.js>
            let output_did_js_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did.js");
            let content =
                ensure_trailing_newline(candid_parser::bindings::javascript::compile(&env, &ty));
            std::fs::write(&output_did_js_path, content).with_context(|| {
                format!(
                    "Failed to write to {}.",
                    output_did_js_path.to_string_lossy()
                )
            })?;
            eprintln!("  {}", &output_did_js_path.display());

            compile_handlebars_files("js", info, generate_output_dir)?;
        }

        // Motoko
        if bindings.contains(&"mo".to_string()) {
            let output_mo_path = generate_output_dir
                .join(info.get_name())
                .with_extension("mo");
            let content =
                ensure_trailing_newline(candid_parser::bindings::motoko::compile(&env, &ty));
            std::fs::write(&output_mo_path, content).with_context(|| {
                format!("Failed to write to {}.", output_mo_path.to_string_lossy())
            })?;
            eprintln!("  {}", &output_mo_path.display());
        }

        // Candid
        if bindings.contains(&"did".to_string()) {
            let output_did_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did");
            dfx_core::fs::copy(&did_from_build, &output_did_path)?;
            dfx_core::fs::set_permissions_readwrite(&output_did_path)?;
            eprintln!("  {}", &output_did_path.display());
        }

        Ok(())
    }

    /// TODO: It is called too many times. It caches data in `env.imports`, but better not to call repeatedly anyway.
    #[context("Failed to find imports for canister '{}'.", info.get_name())]
    fn read_dependencies(
        &self,
        env: &dyn Environment,
        pool: &CanisterPool,
        info: &CanisterInfo,
        cache: &dyn Cache,
    ) -> DfxResult {
        #[context("Failed recursive dependency detection at {}.", parent)]
        fn read_dependencies_recursive(
            env: &dyn Environment,
            cache: &dyn Cache,
            pool: &CanisterPool,
            parent: &Import,
        ) -> DfxResult {
            if env.get_imports().borrow().nodes().contains_key(&parent) {
                // The item and its descendants are already in the graph.
                return Ok(());
            }
            let parent_node_index = env.get_imports().borrow_mut().update_node(&parent);
    
            let file = match parent {
                Import::Canister(parent_name) => {
                    let parent_canister = pool.get_first_canister_with_name(parent_name).unwrap();
                    let parent_canister_info = parent_canister.get_info();
                    if parent_canister_info.is_motoko() {
                        let motoko_info = parent_canister.get_info().as_info::<MotokoCanisterInfo>()?;
                        Some(motoko_info.get_main_path().canonicalize()?)
                    } else {
                        for child in parent_canister_info.get_dependencies() {
                            read_dependencies_recursive(
                                env,
                                cache,
                                pool,
                                &Import::Canister(child.clone()),
                            )?;
        
                            let child_node = Import::Canister(child.clone());
                            let child_node_index = env.get_imports().borrow_mut().update_node(&child_node);
                            env.get_imports().borrow_mut().update_edge(parent_node_index, child_node_index, ());
                        }
                        return Ok(());
                    }
                }
                Import::FullPath(path) => Some(path.clone()),
                _ => None,
            };
            if let Some(file) = file {
                let mut command = cache.get_binary_command("moc")?;
                let command = command.arg("--print-deps").arg(file);
                let output = command
                    .output()
                    .with_context(|| format!("Error executing {:#?}", command))?;
                let output = String::from_utf8_lossy(&output.stdout);

                for line in output.lines() {
                    let child = Import::try_from(line).context("Failed to create MotokoImport.")?;
                    match &child {
                        Import::Canister(_) | Import::FullPath(_) =>
                            read_dependencies_recursive(env, cache, pool, &child)?,
                        _ => {}
                    }
                    let child_node_index = env.get_imports().borrow_mut().update_node(&child);
                    env.get_imports().borrow_mut().update_edge(parent_node_index, child_node_index, ());
                }
            }
    
            Ok(())
        }
    
        read_dependencies_recursive(
            env,
            cache,
            pool,
            &Import::Canister(info.get_name().to_string()),
        )?;
    
        Ok(())
    }

    fn should_build(
        &self,
        env: &dyn Environment,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        cache: &dyn Cache,
        logger: &Logger,
    ) -> DfxResult<bool> {
        if !canister_info.is_motoko() {
            return Ok(true);    
        }

        let output_wasm_path = canister_info.get_output_wasm_path();

        self.read_dependencies(env, pool, canister_info, cache)?;

        // Check that one of the dependencies is newer than the target:
        if let Ok(wasm_file_metadata) = metadata(output_wasm_path) {
            let wasm_file_time = match wasm_file_metadata.modified() {
                Ok(wasm_file_time) => wasm_file_time,
                Err(_) => {
                    return Ok(true); // need to compile
                }
            };
            let imports = env.get_imports().borrow();
            let start = if let Some(node_index) = imports
                .nodes()
                .get(&Import::Canister(canister_info.get_name().to_string()))
            {
                *node_index
            } else {
                panic!("programming error");
            };
            let mut import_iter = Bfs::new(&imports.graph(), start);
            let mut top_level = true; // link to our main Canister with `.wasm`
            loop {
                if let Some(import) = import_iter.next(&imports.graph()) {
                    let top_level_cur = top_level;
                    top_level = false;
                    let subnode = &imports.graph()[import];
                    if top_level_cur {
                        assert!(
                            matches!(subnode, Import::Canister(_)),
                            "the top-level import must be a canister"
                        );
                    }
                    let imported_file = match subnode {
                        Import::Canister(canister_name) => {
                            if let Some(canister) =
                                pool.get_first_canister_with_name(canister_name.as_str())
                            {
                                let main_file = if top_level_cur {
                                    if let Some(main_file) = canister.get_info().get_main_file() {
                                        main_file.to_path_buf()
                                    } else {
                                        continue;
                                    }
                                } else {
                                    canister.get_info().get_service_idl_path()
                                };
                                Some(main_file)
                            } else {
                                None
                            }
                        }
                        Import::Ic(_canister_id) => {
                            continue;
                        }
                        Import::Lib(_path) => {
                            // Skip libs, all changes by package managers don't modify existing directories but create new ones.
                            continue;
                        }
                        Import::FullPath(full_path) => {
                            Some(full_path.clone())
                        }
                    };
                    if let Some(imported_file) = imported_file {
                        let imported_file_metadata = metadata(&imported_file)?;
                        let imported_file_time = imported_file_metadata.modified()?;
                        if imported_file_time > wasm_file_time {
                            break;
                        };
                    };
                } else {
                    trace!(
                        logger,
                        "Canister {} already compiled.",
                        canister_info.get_name()
                    );
                    return Ok(false);
                }
            }
        };

        Ok(true)
    }

    /// Get the path to the provided candid file for the canister.
    /// No need to guarantee the file exists, as the caller will handle that.
    fn get_candid_path(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<PathBuf>;
}

fn compile_handlebars_files(
    lang: &str,
    info: &CanisterInfo,
    generate_output_dir: &Path,
) -> DfxResult {
    // index.js
    let mut language_bindings = crate::util::assets::language_bindings()
        .context("Failed to get language bindings archive.")?;
    for f in language_bindings
        .entries()
        .context("Failed to read language bindings archive entries.")?
    {
        let mut file = f.context("Failed to read language bindings archive entry.")?;

        let pathname: PathBuf = file
            .path()
            .context("Failed to read language bindings entry path name.")?
            .to_path_buf();
        let file_extension = format!("{}.hbs", lang);
        let is_template = pathname
            .to_str()
            .map_or(false, |name| name.ends_with(&file_extension));

        if is_template {
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents)
                .context("Failed to read language bindings archive file content.")?;

            // create the handlebars registry
            let handlebars = Handlebars::new();

            let mut data: BTreeMap<String, &String> = BTreeMap::new();

            let canister_name = &info.get_name().to_string();
            let canister_name_ident = &canister_name.replace('-', "_");

            let node_compatibility = info.get_declarations_config().node_compatibility;

            // Insert only if node outputs are specified
            let actor_export = if node_compatibility {
                // leave empty for nodejs
                "".to_string()
            } else {
                format!(
                    r#"

export const {canister_name_ident} = canisterId ? createActor(canisterId) : undefined;"#,
                )
                .to_string()
            };

            data.insert("canister_name".to_string(), canister_name);
            data.insert("canister_name_ident".to_string(), canister_name_ident);
            data.insert("actor_export".to_string(), &actor_export);

            // Switches to prefixing the canister id with the env variable for frontend declarations as new default
            let process_string_prefix: String = match &info.get_declarations_config().env_override {
                Some(s) => format!(r#""{}""#, s.clone()),
                None => {
                    format!(
                        "process.env.{}{}",
                        "CANISTER_ID_",
                        &canister_name_ident.to_ascii_uppercase(),
                    )
                }
            };

            data.insert(
                "canister_name_process_env".to_string(),
                &process_string_prefix,
            );

            let new_file_contents = handlebars.render_template(&file_contents, &data).unwrap();
            let new_path = generate_output_dir.join(pathname.with_extension(""));
            std::fs::write(&new_path, new_file_contents)
                .with_context(|| format!("Failed to write to {}.", new_path.display()))?;
        }
    }

    Ok(())
}

// TODO: this function was copied from src/lib/models/canister.rs
fn ensure_trailing_newline(s: String) -> String {
    if s.ends_with('\n') {
        s
    } else {
        let mut s = s;
        s.push('\n');
        s
    }
}

/// Execute a command and return its output bytes.
/// If the catch_output is false, the return bytes will always be empty.
pub fn execute_command(
    command: &str,
    vars: &[Env<'_>],
    cwd: &Path,
    catch_output: bool,
) -> DfxResult<Vec<u8>> {
    // No commands, noop.
    if command.is_empty() {
        return Ok(vec![]);
    }
    let words = shell_words::split(command)
        .with_context(|| format!("Cannot parse command '{}'.", command))?;
    let canonical_result = dfx_core::fs::canonicalize(&cwd.join(&words[0]));
    let mut cmd = if words.len() == 1 && canonical_result.is_ok() {
        // If the command is a file, execute it directly.
        let file = canonical_result.unwrap();
        Command::new(file)
    } else {
        // Execute the command in `sh -c` to allow pipes.
        let mut sh_cmd = Command::new("sh");
        sh_cmd.args(["-c", command]);
        sh_cmd
    };

    if !catch_output {
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    for (key, value) in vars {
        cmd.env(key.as_ref(), value);
    }
    let output = cmd
        .output()
        .with_context(|| format!("Error executing custom build step {cmd:#?}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(DfxError::new(BuildError::CustomToolError(
            output.status.code(),
        )))
    }
}

pub fn run_command(command: &str, vars: &[Env<'_>], cwd: &Path) -> DfxResult<()> {
    execute_command(command, vars, cwd, false)?;
    Ok(())
}

pub fn command_output(command: &str, vars: &[Env<'_>], cwd: &Path) -> DfxResult<Vec<u8>> {
    execute_command(command, vars, cwd, true)
}

type Env<'a> = (Cow<'static, str>, Cow<'a, OsStr>);

pub fn get_and_write_environment_variables<'a>(
    info: &CanisterInfo,
    network_name: &'a str,
    pool: &'a CanisterPool,
    dependencies: &[CanisterId],
    write_path: Option<&Path>,
) -> DfxResult<Vec<Env<'a>>> {
    // should not return Err unless write_environment_variables does
    use Cow::*;
    let mut vars = vec![
        (
            Borrowed("DFX_VERSION"),
            Borrowed(dfx_version_str().as_ref()),
        ),
        (Borrowed("DFX_NETWORK"), Borrowed(network_name.as_ref())),
    ];
    for dep in dependencies {
        let canister = if let Some(canister) = pool.get_canister(dep) {
            canister
        } else {
            continue; // TODO: crude hack to prevent backtrace
        };
        if let Some(candid_path) = canister.get_info().get_remote_candid_if_remote() {
            vars.push((
                Owned(format!(
                    "CANISTER_CANDID_PATH_{}",
                    canister.get_name().replace('-', "_").to_ascii_uppercase()
                )),
                Owned(candid_path.as_os_str().to_owned()),
            ));
        } else if let Some(output) = canister.get_build_output() {
            let candid_path = match &output.idl {
                IdlBuildOutput::File(p) => p.as_os_str(),
            };

            vars.push((
                Owned(format!(
                    "CANISTER_CANDID_PATH_{}",
                    canister.get_name().replace('-', "_").to_ascii_uppercase()
                )),
                Borrowed(candid_path),
            ));
        }
    }
    for canister in pool.get_canister_list() {
        // Don't try to add `deploy: false` canisters:
        if let Some(canister_id) = canister.get_info().get_canister_id_option() {
            vars.push((
                Owned(format!(
                    "CANISTER_ID_{}",
                    canister.get_name().replace('-', "_").to_ascii_uppercase(),
                )),
                Owned(canister_id.to_text().into()),
            ));
        }
    }
    if let Ok(id) = info.get_canister_id() {
        vars.push((Borrowed("CANISTER_ID"), Owned(format!("{}", id).into())));
    }
    if let Some(path) = info.get_output_idl_path() {
        vars.push((Borrowed("CANISTER_CANDID_PATH"), Owned(path.into())))
    }

    if let Some(write_path) = write_path {
        write_environment_variables(&vars, write_path)?;
    }
    Ok(vars)
}

fn write_environment_variables(vars: &[Env<'_>], write_path: &Path) -> DfxResult {
    const START_TAG: &str = "\n# DFX CANISTER ENVIRONMENT VARIABLES";
    const END_TAG: &str = "\n# END DFX CANISTER ENVIRONMENT VARIABLES";
    let mut write_string = String::from(START_TAG);
    for (var, val) in vars {
        if let Some(val) = val.to_str() {
            write!(write_string, "\n{var}='{val}'").unwrap();
        }
    }
    write_string.push_str(END_TAG);
    if write_path.try_exists()? {
        // modify the existing file
        let mut existing_file = fs::read_to_string(write_path)?;
        let start_pos = existing_file.rfind(START_TAG);
        if let Some(start_pos) = start_pos {
            // the file exists and already contains our variables, modify only that section
            let end_pos = existing_file[start_pos + START_TAG.len()..].find(END_TAG);
            if let Some(end_pos) = end_pos {
                // the section is correctly formed
                let end_pos = end_pos + END_TAG.len() + start_pos + START_TAG.len();
                existing_file.replace_range(start_pos..end_pos, &write_string);
                dfx_core::fs::write(write_path, existing_file)?;
                return Ok(());
            } else {
                // the file has been edited, so we don't know how much to delete, so we append instead
            }
        }
        // append to the existing file
        existing_file.push_str(&write_string);
        dfx_core::fs::write(write_path, existing_file)?;
    } else {
        // no existing file, okay to clobber
        dfx_core::fs::write(write_path, write_string)?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct BuildConfig {
    profile: Profile,
    pub build_mode_check: bool,
    pub network_name: String,
    pub network_is_playground: bool,

    /// The root of all IDL files.
    pub idl_root: PathBuf,
    /// The root for all language server files.
    pub lsp_root: PathBuf,
    /// The root for all build files.
    pub build_root: PathBuf,
    /// If only a subset of canisters should be built, then user_specified_canisters contains these canisters' names.
    /// If all canisters should be built, then this is None.
    pub user_specified_canisters: Option<Vec<String>>,
    /// If environment variables should be output to a `.env` file, `env_file` is set to its path.
    pub env_file: Option<PathBuf>,
}

impl BuildConfig {
    #[context("Failed to create build config.")]
    pub fn from_config(config: &Config, network_is_playground: bool) -> DfxResult<Self> {
        let config_intf = config.get_config();
        let network_name = util::network_to_pathcompat(&get_network_context()?);
        let network_root = config.get_temp_path()?.join(&network_name);
        let canister_root = network_root.join("canisters");

        Ok(BuildConfig {
            network_name,
            network_is_playground,
            profile: config_intf.profile.unwrap_or(Profile::Debug),
            build_mode_check: false,
            build_root: canister_root.clone(),
            idl_root: canister_root.join("idl/"), // TODO: possibly move to `network_root.join("idl/")`
            lsp_root: network_root.join("lsp/"),
            user_specified_canisters: None,
            env_file: config.get_output_env_file(None)?,
        })
    }

    pub fn with_build_mode_check(self, build_mode_check: bool) -> Self {
        Self {
            build_mode_check,
            ..self
        }
    }

    pub fn with_canisters_to_build(self, canisters: Vec<String>) -> Self {
        Self {
            user_specified_canisters: Some(canisters),
            ..self
        }
    }

    pub fn with_env_file(self, env_file: Option<PathBuf>) -> Self {
        Self { env_file, ..self }
    }
}

pub struct BuilderPool {
    builders: BTreeMap<&'static str, Arc<dyn CanisterBuilder>>,
}

impl BuilderPool {
    #[context("Failed to create new builder pool.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        let builders = BTreeMap::from([
            (
                "assets",
                Arc::new(assets::AssetsBuilder::new(env)?) as Arc<dyn CanisterBuilder>,
            ),
            ("custom", Arc::new(custom::CustomBuilder::new(env)?)),
            ("motoko", Arc::new(motoko::MotokoBuilder::new(env)?)),
            ("rust", Arc::new(rust::RustBuilder::new(env)?)),
            ("pull", Arc::new(pull::PullBuilder::new(env)?)),
        ]);

        Ok(Self { builders })
    }

    pub fn get(&self, info: &CanisterInfo) -> Arc<dyn CanisterBuilder> {
        self.builders[info.get_type_specific_properties().name()].clone()
    }
}
