use crate::config::dfinity::{Config, Profile};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use crate::lib::models::canister::CanisterPool;
use crate::lib::provider::get_network_context;
use crate::util::check_candid_file;

use anyhow::{bail, Context};
use ic_types::principal::Principal as CanisterId;
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
}

/// A stateless canister builder. This is meant to not keep any state and be passed everything.
pub trait CanisterBuilder {
    /// Returns true if this builder supports building the canister.
    fn supports(&self, info: &CanisterInfo) -> bool;

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
            let generate_output_dir = generate_output_dir.canonicalize()?;
            if !generate_output_dir.starts_with(info.get_workspace_root()) {
                bail!(
                    "Directory at '{}' is outside the workspace root.",
                    generate_output_dir.as_path().display()
                );
            }
            std::fs::remove_dir_all(&generate_output_dir)
                .context(format!("Failed to remove dir: {:?}", &generate_output_dir))?;
        }
        std::fs::create_dir_all(&generate_output_dir)
            .context(format!("Failed to create dir: {:?}", &generate_output_dir))?;

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
            std::fs::write(&output_did_ts_path, content)?;
            eprintln!("  {}", &output_did_ts_path.display());
        }

        // Javascript
        if bindings.contains(&"js".to_string()) {
            // <canister.did.js>
            let output_did_js_path = generate_output_dir
                .join(info.get_name())
                .with_extension("did.js");
            let content = ensure_trailing_newline(candid::bindings::javascript::compile(&env, &ty));
            std::fs::write(&output_did_js_path, content)?;
            eprintln!("  {}", &output_did_js_path.display());

            // index.js
            let mut language_bindings = crate::util::assets::language_bindings()?;
            for f in language_bindings.entries()? {
                let mut file = f?;
                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents)?;

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
                std::fs::write(&index_js_path, new_file_contents)?;
                eprintln!("  {}", &index_js_path.display());
            }
        }

        // Motoko
        if bindings.contains(&"mo".to_string()) {
            let output_mo_path = generate_output_dir
                .join(info.get_name())
                .with_extension("mo");
            let content = ensure_trailing_newline(candid::bindings::motoko::compile(&env, &ty));
            std::fs::write(&output_mo_path, content)?;
            eprintln!("  {}", &output_mo_path.display());
        }

        // Candid, delete if not required
        if !bindings.contains(&"did".to_string()) {
            std::fs::remove_file(generated_idl_path)?;
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

#[derive(Clone)]
pub struct BuildConfig {
    profile: Profile,
    pub build_mode_check: bool,
    pub network_name: String,

    /// The root of all IDL files.
    pub idl_root: PathBuf,
    /// The root for all build files.
    pub build_root: PathBuf,
}

impl BuildConfig {
    pub fn from_config(config: &Config) -> DfxResult<Self> {
        let config_intf = config.get_config();
        let network_name = get_network_context()?;
        let build_root = config.get_temp_path().join(&network_name);
        let build_root = build_root.join("canisters");

        Ok(BuildConfig {
            network_name,
            profile: config_intf.profile.unwrap_or(Profile::Debug),
            build_mode_check: false,
            build_root: build_root.clone(),
            idl_root: build_root.join("idl/"),
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
    builders: Vec<Arc<dyn CanisterBuilder>>,
}

impl BuilderPool {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        let builders: Vec<Arc<dyn CanisterBuilder>> = vec![
            Arc::new(assets::AssetsBuilder::new(env)?),
            Arc::new(custom::CustomBuilder::new(env)?),
            Arc::new(motoko::MotokoBuilder::new(env)?),
            Arc::new(rust::RustBuilder::new(env)?),
        ];

        Ok(Self { builders })
    }

    pub fn get(&self, info: &CanisterInfo) -> Option<Arc<dyn CanisterBuilder>> {
        self.builders
            .iter()
            .find(|builder| builder.supports(info))
            .map(|x| Arc::clone(x))
    }
}
