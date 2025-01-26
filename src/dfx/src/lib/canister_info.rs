#![allow(dead_code)]
use crate::lib::error::DfxResult;
use crate::lib::metadata::config::CanisterMetadataConfig;

use anyhow::{anyhow, bail, Context};
use candid::Principal as CanisterId;
use candid::Principal;
use core::panic;
use dfx_core::config::model::dfinity::{
    CanisterDeclarationsConfig, CanisterMetadataSection, CanisterTypeProperties, Config, Pullable,
    TechStack, WasmOptLevel,
};
use dfx_core::fs::canonicalize;
use dfx_core::network::provider::get_network_context;
use dfx_core::util;
use fn_error_context::context;
use std::path::{Path, PathBuf};
use url::Url;

pub mod assets;
pub mod custom;
pub mod motoko;
pub mod pull;
pub mod rust;
use crate::lib::deps::get_candid_path_in_project;

pub trait CanisterInfoFactory {
    fn create(info: &CanisterInfo) -> DfxResult<Self>
    where
        Self: std::marker::Sized;
}

/// Information about a canister project (source code, destination, etc).
#[derive(Debug)]
pub struct CanisterInfo {
    name: String,

    declarations_config: CanisterDeclarationsConfig,
    remote_id: Option<Principal>, // id on the currently selected network
    remote_candid: Option<PathBuf>, // always exists if the field is configured

    workspace_root: PathBuf,
    output_root: PathBuf, // <project dir>/.dfx/<network>/canisters/<canister>

    canister_id: Option<CanisterId>,

    packtool: Option<String>,
    args: Option<String>,
    type_specific: CanisterTypeProperties,

    dependencies: Vec<String>,
    pre_install: Vec<String>,
    post_install: Vec<String>,
    main: Option<PathBuf>,
    shrink: Option<bool>,
    optimize: Option<WasmOptLevel>,
    metadata: CanisterMetadataConfig,
    pullable: Option<Pullable>,
    pull_dependencies: Vec<(String, CanisterId)>,
    tech_stack: Option<TechStack>,
    gzip: bool,
    init_arg: Option<String>,
    init_arg_file: Option<String>,
    output_idl_path: PathBuf,
}

impl CanisterInfo {
    #[context("Failed to load canister info for '{}'.", name)]
    pub fn load(
        config: &Config,
        name: &str,
        canister_id: Option<CanisterId>,
    ) -> DfxResult<CanisterInfo> {
        let workspace_root = config.get_path().parent().unwrap();
        let build_defaults = config.get_config().get_defaults().get_build();
        let network_name = get_network_context()?;
        let build_root = config
            .get_temp_path()?
            .join(util::network_to_pathcompat(&network_name))
            .join("canisters");
        std::fs::create_dir_all(&build_root)
            .with_context(|| format!("Failed to create {}.", build_root.to_string_lossy()))?;

        let canister_map = config
            .get_config()
            .canisters
            .as_ref()
            .ok_or_else(|| anyhow!("No canisters in the configuration file."))?;

        let canister_config = canister_map
            .get(name)
            .ok_or_else(|| anyhow!("Cannot find canister '{}',", name.to_string()))?;

        let dependencies = canister_config.dependencies.clone();

        let mut pull_dependencies = vec![];

        for dep in &dependencies {
            let dep_config = canister_map.get(dep).ok_or_else(|| {
                anyhow!(
                    "Cannot find canister '{}' which is a dependency of '{}'",
                    dep,
                    name.to_string()
                )
            })?;

            if let CanisterTypeProperties::Pull { id } = dep_config.type_specific {
                pull_dependencies.push((dep.to_string(), id))
            }
        }

        let declarations_config_pre = canister_config.declarations.clone();

        let remote_id = canister_config
            .remote
            .as_ref()
            .and_then(|remote| remote.id.get(&network_name))
            .copied();
        let remote_candid = canister_config.remote.as_ref().and_then(|r| {
            r.candid
                .as_ref()
                .and_then(|candid| canonicalize(candid).ok())
        });

        // Fill the default config values if None provided
        let declarations_config = CanisterDeclarationsConfig {
            output: declarations_config_pre
                .output
                .or_else(|| Some(workspace_root.join("src/declarations").join(name))),
            bindings: declarations_config_pre
                .bindings
                .or_else(|| Some(vec!["js".to_string(), "ts".to_string(), "did".to_string()])),
            env_override: declarations_config_pre.env_override,
            node_compatibility: declarations_config_pre.node_compatibility,
        };

        let output_root = build_root.join(name);

        let output_idl_path: PathBuf =
            if let (Some(_id), Some(candid)) = (&remote_id, &remote_candid) {
                workspace_root.join(candid)
            } else {
                match &canister_config.type_specific {
                    CanisterTypeProperties::Rust {
                        package: _,
                        crate_name: _,
                        candid,
                    } => workspace_root.join(candid),
                    CanisterTypeProperties::Assets { .. } => output_root.join("assetstorage.did"),
                    CanisterTypeProperties::Custom {
                        wasm: _,
                        candid,
                        build: _,
                    } => {
                        if Url::parse(candid).is_ok() {
                            output_root.join(name).with_extension("did")
                        } else {
                            workspace_root.join(candid)
                        }
                    }
                    CanisterTypeProperties::Motoko => output_root.join(name).with_extension("did"),
                    CanisterTypeProperties::Pull { id } => {
                        get_candid_path_in_project(workspace_root, id)
                    }
                }
            };

        let type_specific = canister_config.type_specific.clone();

        let args = match &canister_config.args {
            Some(args) if !args.is_empty() => canister_config.args.clone(),
            _ => build_defaults.get_args(),
        };

        let pre_install = canister_config.pre_install.clone().into_vec();
        let post_install = canister_config.post_install.clone().into_vec();
        let metadata = CanisterMetadataConfig::new(&canister_config.metadata, &network_name);

        let gzip = canister_config.gzip.unwrap_or(false);
        let init_arg = canister_config.init_arg.clone();
        let init_arg_file = canister_config.init_arg_file.clone();

        let canister_info = CanisterInfo {
            name: name.to_string(),
            declarations_config,
            remote_id,
            remote_candid,
            workspace_root: workspace_root.to_path_buf(),
            output_root,
            canister_id,
            packtool: build_defaults.get_packtool(),
            args,
            type_specific,
            dependencies,
            pre_install,
            post_install,
            main: canister_config.main.clone(),
            shrink: canister_config.shrink,
            optimize: canister_config.optimize,
            metadata,
            pullable: canister_config.pullable.clone(),
            tech_stack: canister_config.tech_stack.clone(),
            pull_dependencies,
            gzip,
            init_arg,
            init_arg_file,
            output_idl_path,
        };

        Ok(canister_info)
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn get_declarations_config(&self) -> &CanisterDeclarationsConfig {
        &self.declarations_config
    }
    pub fn is_remote(&self) -> bool {
        self.remote_id.is_some()
    }
    pub fn get_remote_id(&self) -> Option<Principal> {
        self.remote_id
    }
    pub fn get_remote_candid(&self) -> Option<PathBuf> {
        self.remote_candid.as_ref().cloned()
    }
    pub fn get_remote_candid_if_remote(&self) -> Option<PathBuf> {
        if self.remote_id.is_some() {
            self.get_remote_candid()
        } else {
            None
        }
    }
    pub fn get_workspace_root(&self) -> &Path {
        &self.workspace_root
    }
    pub fn get_output_root(&self) -> &Path {
        &self.output_root
    }

    #[context("Failed to get canister id for '{}'.", self.name)]
    pub fn get_canister_id(&self) -> DfxResult<CanisterId> {
        match &self.canister_id {
            Some(canister_id) => Ok(*canister_id),
            None => {
                // If we get here, it means there is a logic error in the code.
                // It's not because the user did anything in the wrong order.
                // We need the network type (ephemeral/persistent) in order to load
                // the canister id, so we can't load it here.
                panic!("It is only valid to call get_canister_id after setting the canister id.");
            }
        }
    }

    pub fn get_dependencies(&self) -> &[String] {
        &self.dependencies
    }

    pub fn get_main_file(&self) -> Option<&Path> {
        self.main.as_deref()
    }

    pub fn get_packtool(&self) -> &Option<String> {
        &self.packtool
    }

    pub fn get_pre_install(&self) -> &[String] {
        &self.pre_install
    }

    pub fn get_post_install(&self) -> &[String] {
        &self.post_install
    }

    pub fn get_args(&self) -> &Option<String> {
        &self.args
    }

    pub fn get_shrink(&self) -> Option<bool> {
        self.shrink
    }

    pub fn get_optimize(&self) -> Option<WasmOptLevel> {
        // Cycles defaults to O3, Size defaults to Oz
        self.optimize.map(|level| match level {
            WasmOptLevel::Cycles => WasmOptLevel::O3,
            WasmOptLevel::Size => WasmOptLevel::Oz,
            other => other,
        })
    }

    /// Path to the wasm module in .dfx that will be install.
    pub fn get_build_wasm_path(&self) -> PathBuf {
        let mut gzip_original = false;
        if let CanisterTypeProperties::Custom { wasm, .. } = &self.type_specific {
            if wasm.ends_with(".gz") {
                gzip_original = true;
            }
        } else if self.is_assets() {
            gzip_original = true;
        }
        let ext = if self.gzip || gzip_original {
            "wasm.gz"
        } else {
            "wasm"
        };
        self.output_root.join(&self.name).with_extension(ext)
    }

    /// Path to the candid file which contains no init types.
    ///
    /// To be imported by dependents.
    pub fn get_service_idl_path(&self) -> PathBuf {
        self.output_root.join("service.did")
    }

    /// Path to the candid file which contains init types.
    ///
    /// To be used when installing the canister.
    pub fn get_constructor_idl_path(&self) -> PathBuf {
        self.output_root.join("constructor.did")
    }

    /// Path to the init_args.txt file which only contains init types.
    ///
    pub fn get_init_args_txt_path(&self) -> PathBuf {
        self.output_root.join("init_args.txt")
    }

    pub fn get_index_js_path(&self) -> PathBuf {
        self.output_root.join("index").with_extension("js")
    }

    /// Path to the candid file from canister builder which should contain init types.
    ///
    /// To be separated into service.did and init_args.
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }

    #[context("Failed to create <Type>CanisterInfo for canister '{}'.", self.name, )]
    pub fn as_info<T: CanisterInfoFactory>(&self) -> DfxResult<T> {
        T::create(self)
    }

    pub fn get_type_specific_properties(&self) -> &CanisterTypeProperties {
        &self.type_specific
    }

    pub fn is_motoko(&self) -> bool {
        matches!(self.type_specific, CanisterTypeProperties::Motoko { .. })
    }

    pub fn is_custom(&self) -> bool {
        matches!(self.type_specific, CanisterTypeProperties::Custom { .. })
    }

    pub fn is_rust(&self) -> bool {
        matches!(self.type_specific, CanisterTypeProperties::Rust { .. })
    }

    pub fn is_assets(&self) -> bool {
        matches!(self.type_specific, CanisterTypeProperties::Assets { .. })
    }

    pub fn is_pull(&self) -> bool {
        matches!(self.type_specific, CanisterTypeProperties::Pull { .. })
    }

    pub fn get_metadata(&self, name: &str) -> Option<&CanisterMetadataSection> {
        self.metadata.get(name)
    }

    pub fn metadata(&self) -> &CanisterMetadataConfig {
        &self.metadata
    }

    pub fn get_pullable(&self) -> Option<Pullable> {
        self.pullable.clone()
    }

    pub fn get_pull_dependencies(&self) -> &[(String, CanisterId)] {
        &self.pull_dependencies
    }

    pub fn get_tech_stack(&self) -> Option<&TechStack> {
        self.tech_stack.as_ref()
    }

    pub fn get_gzip(&self) -> bool {
        self.gzip
    }

    /// Get the init arg from the dfx.json configuration.
    ///
    /// If the `init_arg` field is defined, it will be returned.
    /// If the `init_arg_file` field is defined, the content of the file will be returned.
    /// If both fields are defined, an error will be returned.
    /// If neither field is defined, `None` will be returned.
    pub fn get_init_arg(&self) -> DfxResult<Option<String>> {
        let init_arg_value = match (&self.init_arg, &self.init_arg_file) {
            (Some(_), Some(_)) => {
                bail!("At most one of the fields 'init_arg' and 'init_arg_file' should be defined in `dfx.json`.
Please remove one of them or leave both undefined.");
            }
            (Some(arg), None) => Some(arg.clone()),
            (None, Some(arg_file)) => {
                // The file path is relative to the workspace root.
                let absolute_path = self.get_workspace_root().join(arg_file);
                let content = dfx_core::fs::read_to_string(&absolute_path)?;
                Some(content)
            }
            (None, None) => None,
        };

        Ok(init_arg_value)
    }
}
