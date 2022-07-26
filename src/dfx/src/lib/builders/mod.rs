use crate::config::dfinity::{Config, Profile};
use crate::config::dfx_version_str;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use crate::lib::models::canister::CanisterPool;
use crate::lib::provider::get_network_context;
use crate::util::{self, check_candid_file};

use anyhow::{bail, Context};
use candid::Principal as CanisterId;
use fn_error_context::context;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

mod assets;
mod custom;
mod motoko;
mod rust;

#[derive(Debug)]
pub enum WasmBuildOutput {
    // Wasm(Vec<u8>),
    File(PathBuf),
}

#[derive(Debug)]
pub enum IdlBuildOutput {
    // IDLProg(IDLProg),
    File(PathBuf),
}

/// The output of a build.
pub struct BuildOutput {
    pub canister_id: CanisterId,
    pub wasm: WasmBuildOutput,
    pub idl: IdlBuildOutput,
    pub add_candid_service_metadata: bool,
}

/// A stateless canister builder. This is meant to not keep any state and be passed everything.
pub trait CanisterBuilder {
    /// Returns the dependencies of this canister, if any. This should not be a transitive
    /// list.
    fn get_dependencies(
        &self,
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
            let generate_output_dir = generate_output_dir.canonicalize().with_context(|| {
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
        std::fs::create_dir_all(&generate_output_dir).with_context(|| {
            format!(
                "Failed to create dir: {}",
                generate_output_dir.to_string_lossy()
            )
        })?;

        let generated_idl_path = self.generate_idl(pool, info, config)?;

        let (env, ty) = check_candid_file(generated_idl_path.as_path())?;

        let bindings = info
            .get_declarations_config()
            .bindings
            .as_ref()
            .context("`bindings` must not be None")?;

        if bindings.is_empty() {
            eprintln!("`{}.declarations.bindings` in dfx.json was set to be an empty list, so no type declarations will be generated.", &info.get_name());
            return Ok(());
        } else {
            eprintln!(
                "Generating type declarations for canister {}:",
                &info.get_name()
            );
        }

        // Typescript
        if bindings.contains(&"ts".to_string()) {
            let output_did_ts_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did.d.ts");
            let content = ensure_trailing_newline(candid::bindings::typescript::compile(&env, &ty));
            std::fs::write(&output_did_ts_path, content).with_context(|| {
                format!(
                    "Failed to write to {}.",
                    output_did_ts_path.to_string_lossy()
                )
            })?;
            eprintln!("  {}", &output_did_ts_path.display());
        }

        // Javascript
        if bindings.contains(&"js".to_string()) {
            // <canister.did.js>
            let output_did_js_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did.js");
            let content = ensure_trailing_newline(candid::bindings::javascript::compile(&env, &ty));
            std::fs::write(&output_did_js_path, content).with_context(|| {
                format!(
                    "Failed to write to {}.",
                    output_did_js_path.to_string_lossy()
                )
            })?;
            eprintln!("  {}", &output_did_js_path.display());

            // index.js
            let mut language_bindings = crate::util::assets::language_bindings()
                .context("Failed to get language bindings archive.")?;
            for f in language_bindings
                .entries()
                .context("Failed to read language bindings archive entries.")?
            {
                let mut file = f.context("Failed to read language bindings archive entry.")?;
                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents)
                    .context("Failed to read language bindings archive file content.")?;

                let mut new_file_contents = file_contents
                    .replace("{canister_id}", &info.get_canister_id()?.to_text())
                    .replace("{canister_name}", info.get_name());
                new_file_contents = match &info.get_declarations_config().env_override {
                    Some(s) => new_file_contents.replace(
                        "process.env.{canister_name_uppercase}_CANISTER_ID",
                        &format!("\"{}\"", s),
                    ),
                    None => new_file_contents
                        .replace("{canister_name_uppercase}", &info.get_name().to_uppercase()),
                };
                let index_js_path = generate_output_dir.join("index").with_extension("js");
                std::fs::write(&index_js_path, new_file_contents).with_context(|| {
                    format!("Failed to write to {}.", index_js_path.to_string_lossy())
                })?;
                eprintln!("  {}", &index_js_path.display());
            }
        }

        // Motoko
        if bindings.contains(&"mo".to_string()) {
            let output_mo_path = generate_output_dir
                .join(info.get_name())
                .with_extension("mo");
            let content = ensure_trailing_newline(candid::bindings::motoko::compile(&env, &ty));
            std::fs::write(&output_mo_path, content).with_context(|| {
                format!("Failed to write to {}.", output_mo_path.to_string_lossy())
            })?;
            eprintln!("  {}", &output_mo_path.display());
        }

        // Candid, delete if not required
        if !bindings.contains(&"did".to_string()) {
            std::fs::remove_file(&generated_idl_path).with_context(|| {
                format!("Failed to remove {}.", generated_idl_path.to_string_lossy())
            })?;
        } else {
            eprintln!("  {}", &generated_idl_path.display());
        }
        Ok(())
    }

    fn generate_idl(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        Ok(PathBuf::new())
    }
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

type Env<'a> = (Cow<'static, str>, Cow<'a, OsStr>);

pub fn environment_variables<'a>(
    info: &CanisterInfo,
    network_name: &'a str,
    pool: &'a CanisterPool,
    dependencies: &[CanisterId],
) -> Vec<Env<'a>> {
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
        if let Some(output) = canister.get_build_output() {
            let candid_path = match &output.idl {
                IdlBuildOutput::File(p) => p.as_os_str(),
            };

            vars.push((
                Owned(format!("CANISTER_CANDID_PATH_{}", canister.get_name())),
                Borrowed(candid_path),
            ));
        }
    }
    for canister in pool.get_canister_list() {
        vars.push((
            Owned(format!("CANISTER_ID_{}", canister.get_name())),
            Owned(canister.canister_id().to_text().into()),
        ));
    }
    if let Ok(id) = info.get_canister_id() {
        vars.push((Borrowed("CANISTER_ID"), Owned(format!("{}", id).into())));
    }
    if let Some(path) = info.get_output_idl_path() {
        vars.push((Borrowed("CANISTER_CANDID_PATH"), Owned(path.into())))
    }
    vars
}

#[derive(Clone)]
pub struct BuildConfig {
    profile: Profile,
    pub build_mode_check: bool,
    pub network_name: String,

    /// The root of all IDL files.
    pub idl_root: PathBuf,
    /// The root for all language server files.
    pub lsp_root: PathBuf,
    /// The root for all build files.
    pub build_root: PathBuf,
}

impl BuildConfig {
    #[context("Failed to create build config.")]
    pub fn from_config(config: &Config) -> DfxResult<Self> {
        let config_intf = config.get_config();
        let network_name = util::network_to_pathcompat(&get_network_context()?);
        let network_root = config.get_temp_path().join(&network_name);
        let canister_root = network_root.join("canisters");

        Ok(BuildConfig {
            network_name,
            profile: config_intf.profile.unwrap_or(Profile::Debug),
            build_mode_check: false,
            build_root: canister_root.clone(),
            idl_root: canister_root.join("idl/"), // TODO: possibly move to `network_root.join("idl/")`
            lsp_root: network_root.join("lsp/"),
        })
    }

    pub fn with_build_mode_check(self, build_mode_check: bool) -> Self {
        Self {
            build_mode_check,
            ..self
        }
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
        ]);

        Ok(Self { builders })
    }

    pub fn get(&self, info: &CanisterInfo) -> Arc<dyn CanisterBuilder> {
        self.builders[info.get_type_specific_properties().name()].clone()
    }
}
