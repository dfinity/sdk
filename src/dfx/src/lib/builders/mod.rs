use crate::config::dfx_version_str;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::util::command::direct_or_shell_command;
use crate::util::with_suspend_all_spinners;
use anyhow::{bail, Context};
use candid::Principal as CanisterId;
use candid_parser::utils::CandidSource;
use dfx_core::config::model::dfinity::{Config, Profile};
use dfx_core::network::provider::get_network_context;
use dfx_core::util;
use fn_error_context::context;
use handlebars::Handlebars;
use slog::{info, trace, Logger};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

mod assets;
mod custom;
mod motoko;
mod pull;
mod rust;

pub use custom::custom_download;

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
        _env: &dyn Environment,
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
        _env: &dyn Environment,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult {
        Ok(())
    }

    /// Generate type declarations for the canister
    fn generate(
        &self,
        env: &dyn Environment,
        logger: &Logger,
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
                        generate_output_dir.display()
                    )
                })?;
            if !generate_output_dir.starts_with(info.get_workspace_root()) {
                bail!(
                    "Directory at '{}' is outside the workspace root.",
                    generate_output_dir.as_path().display()
                );
            }
            std::fs::remove_dir_all(&generate_output_dir).with_context(|| {
                format!("Failed to remove dir: {}", generate_output_dir.display())
            })?;
        }

        let bindings = info
            .get_declarations_config()
            .bindings
            .as_ref()
            .context("`bindings` must not be None")?;

        if bindings.is_empty() {
            info!(logger, "`{}.declarations.bindings` in dfx.json was set to be an empty list, so no type declarations will be generated.", &info.get_name());
            return Ok(());
        }

        let spinner = env.new_spinner(
            format!(
                "Generating type declarations for canister {}",
                &info.get_name()
            )
            .into(),
        );

        std::fs::create_dir_all(generate_output_dir)
            .with_context(|| format!("Failed to create dir: {}", generate_output_dir.display()))?;

        let did_from_build = self.get_candid_path(env, pool, info, config)?;
        if !did_from_build.exists() {
            bail!("Candid file: {} doesn't exist.", did_from_build.display());
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
            trace!(logger, "  {}", &output_did_ts_path.display());

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
            std::fs::write(&output_did_js_path, content)
                .with_context(|| format!("Failed to write to {}.", output_did_js_path.display()))?;
            trace!(logger, "  {}", &output_did_js_path.display());

            compile_handlebars_files("js", info, generate_output_dir)?;
        }

        // Motoko
        if bindings.contains(&"mo".to_string()) {
            let output_mo_path = generate_output_dir
                .join(info.get_name())
                .with_extension("mo");
            let content =
                ensure_trailing_newline(candid_parser::bindings::motoko::compile(&env, &ty));
            std::fs::write(&output_mo_path, content)
                .with_context(|| format!("Failed to write to {}.", output_mo_path.display()))?;
            trace!(logger, "  {}", &output_mo_path.display());
        }

        // Candid
        if bindings.contains(&"did".to_string()) {
            let output_did_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did");
            dfx_core::fs::copy(&did_from_build, &output_did_path)?;
            dfx_core::fs::set_permissions_readwrite(&output_did_path)?;
            trace!(logger, "  {}", &output_did_path.display());
        }

        spinner.finish_and_clear();
        info!(
            logger,
            "Generated type declarations for canister '{}' to '{}'",
            &info.get_name(),
            generate_output_dir.display()
        );

        Ok(())
    }

    /// Get the path to the provided candid file for the canister.
    /// No need to guarantee the file exists, as the caller will handle that.
    fn get_candid_path(
        &self,
        env: &dyn Environment,
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
            .is_some_and(|name| name.ends_with(&file_extension));

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
    env: &dyn Environment,
    command: &str,
    vars: &[Env<'_>],
    cwd: &Path,
    catch_output: bool,
) -> DfxResult<Vec<u8>> {
    // No commands, noop.
    if command.is_empty() {
        return Ok(vec![]);
    }
    let mut cmd = direct_or_shell_command(command, cwd)?;

    if !catch_output {
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    for (key, value) in vars {
        cmd.env(key.as_ref(), value);
    }
    let output = if catch_output {
        cmd.output()
    } else {
        with_suspend_all_spinners(env, || cmd.output())
    }
    .with_context(|| format!("Error executing custom build step {cmd:#?}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(DfxError::new(BuildError::CustomToolError(
            output.status.code(),
        )))
    }
}

pub fn run_command(
    env: &dyn Environment,
    command: &str,
    vars: &[Env<'_>],
    cwd: &Path,
) -> DfxResult<()> {
    execute_command(env, command, vars, cwd, false)?;
    Ok(())
}

pub fn command_output(
    env: &dyn Environment,
    command: &str,
    vars: &[Env<'_>],
    cwd: &Path,
) -> DfxResult<Vec<u8>> {
    execute_command(env, command, vars, cwd, true)
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
        let canister = pool.get_canister(dep).unwrap();
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
        vars.push((
            Owned(format!(
                "CANISTER_ID_{}",
                canister.get_name().replace('-', "_").to_ascii_uppercase(),
            )),
            Owned(canister.canister_id().to_text().into()),
        ));
    }
    if let Ok(id) = info.get_canister_id() {
        vars.push((Borrowed("CANISTER_ID"), Owned(format!("{}", id).into())));
    }
    vars.push((
        Borrowed("CANISTER_CANDID_PATH"),
        Owned(info.get_output_idl_path().into()),
    ));

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
    pub network_name: String,

    /// The root of all IDL files.
    pub idl_root: PathBuf,
    /// The root for all language server files.
    pub lsp_root: PathBuf,
    /// If only a subset of canisters should be built, then canisters_to_build contains these canisters' names.
    /// If all canisters should be built, then this is None.
    pub canisters_to_build: Option<Vec<String>>,
    /// If environment variables should be output to a `.env` file, `env_file` is set to its path.
    pub env_file: Option<PathBuf>,
}

impl BuildConfig {
    #[context("Failed to create build config.")]
    pub fn from_config(config: &Config) -> DfxResult<Self> {
        let config_intf = config.get_config();
        let network_name = util::network_to_pathcompat(&get_network_context()?);
        let network_root = config.get_temp_path()?.join(&network_name);
        let canister_root = network_root.join("canisters");

        Ok(BuildConfig {
            network_name,
            profile: config_intf.profile.unwrap_or(Profile::Debug),
            idl_root: canister_root.join("idl/"), // TODO: possibly move to `network_root.join("idl/")`
            lsp_root: network_root.join("lsp/"),
            canisters_to_build: None,
            env_file: config.get_output_env_file(None)?,
        })
    }

    pub fn with_canisters_to_build(self, canisters: Vec<String>) -> Self {
        Self {
            canisters_to_build: Some(canisters),
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
