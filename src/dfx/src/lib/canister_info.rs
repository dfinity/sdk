#![allow(dead_code)]
use crate::config::dfinity::{CanisterDeclarationsConfig, CanisterTypeProperties, Config};
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::custom::CustomCanisterInfo;
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::error::DfxResult;
use crate::lib::provider::get_network_context;
use crate::util;

use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use candid::Principal;
use core::panic;
use fn_error_context::context;
use std::path::{Path, PathBuf};

use self::rust::RustCanisterInfo;

pub mod assets;
pub mod custom;
pub mod motoko;
pub mod rust;

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
    post_install: Vec<String>,
    main: Option<PathBuf>,
    shrink: bool,
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
            .get_temp_path()
            .join(util::network_to_pathcompat(&network_name))
            .join("canisters");
        std::fs::create_dir_all(&build_root)
            .with_context(|| format!("Failed to create {}.", build_root.to_string_lossy()))?;

        let canister_map = (&config.get_config().canisters)
            .as_ref()
            .ok_or_else(|| anyhow!("No canisters in the configuration file."))?;

        let canister_config = canister_map
            .get(name)
            .ok_or_else(|| anyhow!("Cannot find canister '{}',", name.to_string()))?;

        let declarations_config_pre = canister_config.declarations.clone();

        let remote_id = canister_config
            .remote
            .as_ref()
            .and_then(|remote| remote.id.get(&network_name))
            .copied();
        let remote_candid = canister_config
            .remote
            .as_ref()
            .and_then(|r| r.candid.as_ref())
            .cloned();

        // Fill the default config values if None provided
        let declarations_config = CanisterDeclarationsConfig {
            output: declarations_config_pre
                .output
                .or_else(|| Some(PathBuf::from("src/declarations").join(name))),
            bindings: declarations_config_pre
                .bindings
                .or_else(|| Some(vec!["js".to_string(), "ts".to_string(), "did".to_string()])),
            env_override: declarations_config_pre.env_override,
            node_compatibility: declarations_config_pre.node_compatibility,
        };

        let output_root = build_root.join(name);

        let type_specific = canister_config.type_specific.clone();

        let args = match &canister_config.args {
            Some(args) if !args.is_empty() => canister_config.args.clone(),
            _ => build_defaults.get_args(),
        };

        let post_install = canister_config.post_install.clone().into_vec();

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
            dependencies: canister_config.dependencies.clone(),
            post_install,
            main: canister_config.main.clone(),
            shrink: canister_config.shrink,
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

    pub fn get_post_install(&self) -> &[String] {
        &self.post_install
    }

    pub fn get_args(&self) -> &Option<String> {
        &self.args
    }

    pub fn get_shrink(&self) -> bool {
        self.shrink
    }

    pub fn get_build_wasm_path(&self) -> PathBuf {
        self.output_root.join(&self.name).with_extension("wasm")
    }

    pub fn get_build_idl_path(&self) -> PathBuf {
        self.output_root.join(&self.name).with_extension("did")
    }

    pub fn get_index_js_path(&self) -> PathBuf {
        self.output_root.join("index").with_extension("js")
    }

    pub fn get_output_idl_path(&self) -> Option<PathBuf> {
        match &self.type_specific {
            CanisterTypeProperties::Motoko { .. } => self
                .as_info::<MotokoCanisterInfo>()
                .map(|x| x.get_output_idl_path().to_path_buf()),
            CanisterTypeProperties::Custom { .. } => self
                .as_info::<CustomCanisterInfo>()
                .map(|x| x.get_output_idl_path().to_path_buf()),
            CanisterTypeProperties::Assets { .. } => self
                .as_info::<AssetsCanisterInfo>()
                .map(|x| x.get_output_idl_path().to_path_buf()),
            CanisterTypeProperties::Rust { .. } => self
                .as_info::<RustCanisterInfo>()
                .map(|x| x.get_output_idl_path().to_path_buf()),
        }
        .ok()
        .or_else(|| self.remote_candid.clone())
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
}
