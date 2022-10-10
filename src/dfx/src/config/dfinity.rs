#![allow(dead_code)]
use crate::lib::bitcoin::adapter::config::BitcoinAdapterLogLevel;
use crate::lib::canister_http::adapter::config::HttpAdapterLogLevel;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::util::{PossiblyStr, SerdeVec};
use crate::{error_invalid_argument, error_invalid_config, error_invalid_data};

use anyhow::{anyhow, Context};
use byte_unit::Byte;
use candid::Principal;
use directories_next::ProjectDirs;
use fn_error_context::context;
use schemars::JsonSchema;
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::default::Default;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const CONFIG_FILE_NAME: &str = "dfx.json";

const EMPTY_CONFIG_DEFAULTS: ConfigDefaults = ConfigDefaults {
    bitcoin: None,
    bootstrap: None,
    build: None,
    canister_http: None,
    replica: None,
};

const EMPTY_CONFIG_DEFAULTS_BUILD: ConfigDefaultsBuild = ConfigDefaultsBuild {
    packtool: None,
    args: None,
};

/// # Remote Canister Configuration
/// This field allows canisters to be marked 'remote' for certain networks.
/// On networks where this canister contains a remote ID, the canister is not deployed.
/// Instead it is assumed to exist already under control of a different project.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ConfigCanistersCanisterRemote {
    /// # Remote Candid File
    /// On networks where this canister is marked 'remote', this candid file is used instead of the one declared in the canister settings.
    pub candid: Option<PathBuf>,

    /// # Network to Remote ID Mapping
    /// This field contains mappings from network names to remote canister IDs (Principals).
    /// For all networks listed here, this canister is considered 'remote'.
    #[schemars(with = "BTreeMap<String, String>")]
    pub id: BTreeMap<String, Principal>,
}

pub const DEFAULT_SHARED_LOCAL_BIND: &str = "127.0.0.1:4943"; // hex for "IC"
pub const DEFAULT_PROJECT_LOCAL_BIND: &str = "127.0.0.1:8000";
pub const DEFAULT_IC_GATEWAY: &str = "https://ic0.app";
pub const DEFAULT_IC_GATEWAY_TRAILING_SLASH: &str = "https://ic0.app/";
pub const DEFAULT_REPLICA_PORT: u16 = 8080;

/// # Canister Configuration
/// Configurations for a single canister.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConfigCanistersCanister {
    /// # Declarations Configuration
    /// Defines which canister interface declarations to generate,
    /// and where to generate them.
    #[serde(default)]
    pub declarations: CanisterDeclarationsConfig,

    /// # Remote Configuration
    /// Used to mark the canister as 'remote' on certain networks.
    #[serde(default)]
    pub remote: Option<ConfigCanistersCanisterRemote>,

    /// # Canister Argument
    /// This field defines a static argument to use when deploying the canister.
    pub args: Option<String>,

    /// # Resource Allocation Settings
    /// Defines initial values for resource allocation settings.
    #[serde(default)]
    pub initialization_values: InitializationValues,

    /// # Dependencies
    /// Defines on which canisters this canister depends on.
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// # Force Frontend URL
    /// Mostly unused.
    /// If this value is not null, a frontend URL is displayed after deployment even if the canister type is not 'asset'.
    pub frontend: Option<BTreeMap<String, String>>,

    /// # Type-Specific Canister Properties
    /// Depending on the canister type, different fields are required.
    /// These are defined in this object.
    #[serde(flatten)]
    pub type_specific: CanisterTypeProperties,

    /// # Post-Install Commands
    /// One or more commands to run post canister installation.
    #[serde(default)]
    pub post_install: SerdeVec<String>,

    /// # Path to Canister Entry Point
    /// Entry point for e.g. Motoko Compiler.
    pub main: Option<PathBuf>,

    /// # Shrink Canister WASM
    /// Whether run `ic-wasm shrink` after building the Canister.
    /// Default is true.
    #[serde(default = "default_as_true")]
    pub shrink: bool,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CanisterTypeProperties {
    /// # Rust-Specific Properties
    Rust {
        /// # Package Name
        /// Name of the rust package that compiles to this canister's WASM.
        package: String,

        /// # Candid File
        /// Path of this canister's candid interface declaration.
        candid: PathBuf,
    },
    /// # Asset-Specific Properties
    Assets {
        /// # Asset Source Folder
        /// Folders from which assets are uploaded.
        source: Vec<PathBuf>,
    },
    /// # Custom-Specific Properties
    Custom {
        /// # WASM Path
        /// Path to WASM to be installed. URLs to a WASM module are also acceptable.
        wasm: String,

        /// # Candid File
        /// Path to this canister's candid interface declaration.  A URL to a candid file is also acceptable.
        candid: String,

        /// # Build Commands
        /// Commands that are executed in order to produce this canister's WASM module.
        /// Expected to produce the WASM in the path specified by the 'wasm' field.
        build: SerdeVec<String>,
    },
    /// # Motoko-Specific Properties
    Motoko,
}

impl CanisterTypeProperties {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust { .. } => "rust",
            Self::Motoko { .. } => "motoko",
            Self::Assets { .. } => "assets",
            Self::Custom { .. } => "custom",
        }
    }
}

/// # Initial Resource Allocations
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct InitializationValues {
    /// # Compute Allocation
    /// Must be a number between 0 and 100, inclusively.
    /// It indicates how much compute power should be guaranteed to this canister, expressed as a percentage of the maximum compute power that a single canister can allocate.
    pub compute_allocation: Option<PossiblyStr<u64>>,

    /// # Memory Allocation
    /// Maximum memory (in bytes) this canister is allowed to occupy.
    #[schemars(with = "Option<u64>")]
    pub memory_allocation: Option<Byte>,

    /// # Freezing Threshold
    /// Freezing threshould of the canister, measured in seconds.
    /// Valid inputs are numbers (seconds) or strings parsable by humantime (e.g. "15days 2min 2s").
    #[serde(with = "humantime_serde")]
    #[schemars(with = "Option<String>")]
    pub freezing_threshold: Option<Duration>,
}

/// # Declarations Configuration
/// Configurations about which canister interface declarations to generate,
/// and where to generate them.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct CanisterDeclarationsConfig {
    /// # Declaration Output Directory
    /// Directory to place declarations for that canister.
    /// Default is 'src/declarations/<canister_name>'.
    pub output: Option<PathBuf>,

    /// # Languages to generate
    /// A list of languages to generate type declarations.
    /// Supported options are 'js', 'ts', 'did', 'mo'.
    /// Default is ['js', 'ts', 'did'].
    pub bindings: Option<Vec<String>>,

    /// # Canister ID ENV Override
    /// A string that will replace process.env.{canister_name_uppercase}_CANISTER_ID
    /// in the 'src/dfx/assets/language_bindings/canister.js' template.
    pub env_override: Option<String>,

    /// # Node compatibility flag
    /// Flag to pre-populate generated declarations with better defaults for various types of projects
    /// Default is false
    #[serde(default)]
    pub node_compatibility: bool,
}

/// # Bitcoin Adapter Configuration
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaultsBitcoin {
    /// # Enable Bitcoin Adapter
    #[serde(default)]
    pub enabled: bool,

    /// # Available Nodes
    /// Addresses of nodes to connect to (in case discovery from seeds is not possible/sufficient).
    #[serde(default)]
    pub nodes: Option<Vec<SocketAddr>>,

    /// # Logging Level
    /// The logging level of the adapter.
    #[serde(default)]
    pub log_level: BitcoinAdapterLogLevel,
}

impl Default for ConfigDefaultsBitcoin {
    fn default() -> Self {
        ConfigDefaultsBitcoin {
            enabled: false,
            nodes: None,
            log_level: BitcoinAdapterLogLevel::Info,
        }
    }
}

/// # HTTP Adapter Configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaultsCanisterHttp {
    /// # Enable HTTP Adapter
    #[serde(default = "default_as_true")]
    pub enabled: bool,

    /// # Logging Level
    /// The logging level of the adapter.
    #[serde(default)]
    pub log_level: HttpAdapterLogLevel,
}

impl Default for ConfigDefaultsCanisterHttp {
    fn default() -> Self {
        ConfigDefaultsCanisterHttp {
            enabled: true,
            log_level: HttpAdapterLogLevel::default(),
        }
    }
}

fn default_as_true() -> bool {
    // sigh https://github.com/serde-rs/serde/issues/368
    true
}

/// # Bootstrap Server Configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaultsBootstrap {
    /// Specifies the IP address that the bootstrap server listens on. Defaults to 127.0.0.1.
    #[serde(default = "default_bootstrap_ip")]
    pub ip: IpAddr,

    /// Specifies the port number that the bootstrap server listens on. Defaults to 8081.
    #[serde(default = "default_bootstrap_port")]
    pub port: u16,

    /// Specifies the maximum number of seconds that the bootstrap server
    /// will wait for upstream requests to complete. Defaults to 30.
    #[serde(default = "default_bootstrap_timeout")]
    pub timeout: u64,
}

pub fn default_bootstrap_ip() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}
pub fn default_bootstrap_port() -> u16 {
    8081
}
pub fn default_bootstrap_timeout() -> u64 {
    30
}

impl Default for ConfigDefaultsBootstrap {
    fn default() -> Self {
        ConfigDefaultsBootstrap {
            ip: default_bootstrap_ip(),
            port: default_bootstrap_port(),
            timeout: default_bootstrap_timeout(),
        }
    }
}

/// # Build Process Configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaultsBuild {
    /// Main command to run the packtool.
    pub packtool: Option<String>,

    /// Arguments for packtool.
    pub args: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReplicaLogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Default for ReplicaLogLevel {
    fn default() -> Self {
        Self::Error
    }
}

impl ReplicaLogLevel {
    pub fn as_ic_starter_string(&self) -> String {
        match self {
            Self::Critical => "critical".to_string(),
            Self::Error => "error".to_string(),
            Self::Warning => "warning".to_string(),
            Self::Info => "info".to_string(),
            Self::Debug => "debug".to_string(),
            Self::Trace => "trace".to_string(),
        }
    }
}

/// # Local Replica Configuration
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaultsReplica {
    /// Port the replica listens on.
    pub port: Option<u16>,

    /// # Subnet Type
    /// Determines the subnet type the replica will run as.
    /// Affects things like cycles accounting, message size limits, cycle limits.
    /// Defaults to 'application'.
    pub subnet_type: Option<ReplicaSubnetType>,

    /// Run replica with the provided log level. Default is 'error'. Debug prints still get displayed
    pub log_level: Option<ReplicaLogLevel>,
}

// Schemars doesn't add the enum value's docstrings. Therefore the explanations have to be up here.
/// # Network Type
/// Type 'ephemeral' is used for networks that are regularly reset.
/// Type 'persistent' is used for networks that last for a long time and where it is preferred that canister IDs get stored in source control.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    // We store ephemeral canister ids in .dfx/{network}/canister_ids.json
    Ephemeral,

    // We store persistent canister ids in canister_ids.json (adjacent to dfx.json)
    Persistent,
}

impl Default for NetworkType {
    // This is just needed for the Default trait on NetworkType,
    // but nothing will ever call it, due to field defaults.
    fn default() -> Self {
        NetworkType::Ephemeral
    }
}

impl NetworkType {
    fn ephemeral() -> Self {
        NetworkType::Ephemeral
    }
    fn persistent() -> Self {
        NetworkType::Persistent
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReplicaSubnetType {
    System,
    Application,
    VerifiedApplication,
}

impl Default for ReplicaSubnetType {
    fn default() -> Self {
        ReplicaSubnetType::Application
    }
}

impl ReplicaSubnetType {
    /// Converts the value to the string expected by ic-starter for its --subnet-type argument
    pub fn as_ic_starter_string(&self) -> String {
        match self {
            ReplicaSubnetType::System => "system".to_string(),
            ReplicaSubnetType::Application => "application".to_string(),
            ReplicaSubnetType::VerifiedApplication => "verified_application".to_string(),
        }
    }
}

/// # Custom Network Configuration
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigNetworkProvider {
    /// The URL(s) this network can be reached at.
    pub providers: Vec<String>,

    /// Persistence type of this network.
    #[serde(default = "NetworkType::persistent")]
    pub r#type: NetworkType,
}

/// # Local Replica Configuration
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigLocalProvider {
    /// Bind address for the webserver.
    /// For the shared local network, the default is 127.0.0.1:4943.
    /// For project-specific local networks, the default is 127.0.0.1:8000.
    pub bind: Option<String>,

    /// Persistence type of this network.
    #[serde(default = "NetworkType::ephemeral")]
    pub r#type: NetworkType,

    pub bitcoin: Option<ConfigDefaultsBitcoin>,
    pub bootstrap: Option<ConfigDefaultsBootstrap>,
    pub canister_http: Option<ConfigDefaultsCanisterHttp>,
    pub replica: Option<ConfigDefaultsReplica>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ConfigNetwork {
    ConfigNetworkProvider(ConfigNetworkProvider),
    ConfigLocalProvider(ConfigLocalProvider),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    // debug is for development only
    Debug,
    // release is for production
    Release,
}

/// Defaults to use on dfx start.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaults {
    pub bitcoin: Option<ConfigDefaultsBitcoin>,
    pub bootstrap: Option<ConfigDefaultsBootstrap>,
    pub build: Option<ConfigDefaultsBuild>,
    pub canister_http: Option<ConfigDefaultsCanisterHttp>,
    pub replica: Option<ConfigDefaultsReplica>,
}

/// # dfx.json
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConfigInterface {
    pub profile: Option<Profile>,

    /// Used to keep track of dfx.json versions.
    pub version: Option<u32>,

    /// # dfx version
    /// Pins the dfx version for this project.
    pub dfx: Option<String>,

    /// Mapping between canisters and their settings.
    pub canisters: Option<BTreeMap<String, ConfigCanistersCanister>>,

    /// Defaults for dfx start.
    pub defaults: Option<ConfigDefaults>,

    /// Mapping between network names and their configurations.
    /// Networks 'ic' and 'local' are implicitly defined.
    pub networks: Option<BTreeMap<String, ConfigNetwork>>,
}

pub type TopLevelConfigNetworks = BTreeMap<String, ConfigNetwork>;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NetworksConfigInterface {
    pub networks: TopLevelConfigNetworks,
}

impl ConfigCanistersCanister {}

#[context("Failed to convert '{}' to a SocketAddress.", s)]
pub fn to_socket_addr(s: &str) -> DfxResult<SocketAddr> {
    match s.to_socket_addrs() {
        Ok(mut a) => match a.next() {
            Some(res) => Ok(res),
            None => Err(DfxError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Empty iterator",
            ))),
        },
        Err(err) => Err(DfxError::new(err)),
    }
}

impl ConfigDefaultsBuild {
    pub fn get_packtool(&self) -> Option<String> {
        match &self.packtool {
            Some(v) if !v.is_empty() => self.packtool.to_owned(),
            _ => None,
        }
    }
    pub fn get_args(&self) -> Option<String> {
        match &self.args {
            Some(v) if !v.is_empty() => self.args.to_owned(),
            _ => None,
        }
    }
}

impl ConfigDefaults {
    pub fn get_build(&self) -> &ConfigDefaultsBuild {
        match &self.build {
            Some(x) => x,
            None => &EMPTY_CONFIG_DEFAULTS_BUILD,
        }
    }
}

impl NetworksConfigInterface {
    pub fn get_network(&self, name: &str) -> Option<&ConfigNetwork> {
        self.networks.get(name)
    }
}

impl ConfigInterface {
    pub fn get_defaults(&self) -> &ConfigDefaults {
        match &self.defaults {
            Some(v) => v,
            _ => &EMPTY_CONFIG_DEFAULTS,
        }
    }

    pub fn get_network(&self, name: &str) -> Option<&ConfigNetwork> {
        self.networks
            .as_ref()
            .and_then(|networks| networks.get(name))
    }

    pub fn get_version(&self) -> u32 {
        self.version.unwrap_or(1)
    }
    pub fn get_dfx(&self) -> Option<String> {
        self.dfx.to_owned()
    }

    /// Return the names of the specified canister and all of its dependencies.
    /// If none specified, return the names of all canisters.
    #[context("Failed to get canisters with their dependencies (for {}).", some_canister.unwrap_or("all canisters"))]
    pub fn get_canister_names_with_dependencies(
        &self,
        some_canister: Option<&str>,
    ) -> DfxResult<Vec<String>> {
        let canister_map = (&self.canisters)
            .as_ref()
            .ok_or_else(|| error_invalid_config!("No canisters in the configuration file."))?;

        let canister_names = match some_canister {
            Some(specific_canister) => {
                let mut names = HashSet::new();
                let mut path = vec![];
                add_dependencies(canister_map, &mut names, &mut path, specific_canister)?;
                names.into_iter().collect()
            }
            None => canister_map.keys().cloned().collect(),
        };

        Ok(canister_names)
    }

    #[context(
        "Failed to figure out if canister '{}' has a remote id on network '{}'.",
        canister,
        network
    )]
    pub fn get_remote_canister_id(
        &self,
        canister: &str,
        network: &str,
    ) -> DfxResult<Option<Principal>> {
        let maybe_principal = (&self.canisters)
            .as_ref()
            .ok_or_else(|| error_invalid_config!("No canisters in the configuration file."))?
            .get(canister)
            .ok_or_else(|| error_invalid_argument!("Canister {} not found in dfx.json", canister))?
            .remote
            .as_ref()
            .and_then(|r| r.id.get(network))
            .copied();
        Ok(maybe_principal)
    }

    #[context(
        "Failed while determining if canister '{}' is remote on network '{}'.",
        canister,
        network
    )]
    pub fn is_remote_canister(&self, canister: &str, network: &str) -> DfxResult<bool> {
        Ok(self.get_remote_canister_id(canister, network)?.is_some())
    }

    #[context("Failed to get compute allocation for '{}'.", canister_name)]
    pub fn get_compute_allocation(&self, canister_name: &str) -> DfxResult<Option<u64>> {
        Ok(self
            .get_canister_config(canister_name)?
            .initialization_values
            .compute_allocation
            .map(|x| x.0))
    }

    #[context("Failed to get memory allocation for '{}'.", canister_name)]
    pub fn get_memory_allocation(&self, canister_name: &str) -> DfxResult<Option<Byte>> {
        Ok(self
            .get_canister_config(canister_name)?
            .initialization_values
            .memory_allocation)
    }

    #[context("Failed to get freezing threshold for '{}'.", canister_name)]
    pub fn get_freezing_threshold(&self, canister_name: &str) -> DfxResult<Option<Duration>> {
        Ok(self
            .get_canister_config(canister_name)?
            .initialization_values
            .freezing_threshold)
    }

    fn get_canister_config(&self, canister_name: &str) -> DfxResult<&ConfigCanistersCanister> {
        let canister_map = self
            .canisters
            .as_ref()
            .ok_or_else(|| error_invalid_config!("No canisters in the configuration file."))?;

        let canister_config = canister_map
            .get(canister_name)
            .with_context(|| format!("Cannot find canister '{canister_name}'."))?;
        Ok(canister_config)
    }
}

#[context("Failed to add dependencies for canister '{}'.", canister_name)]
fn add_dependencies(
    all_canisters: &BTreeMap<String, ConfigCanistersCanister>,
    names: &mut HashSet<String>,
    path: &mut Vec<String>,
    canister_name: &str,
) -> DfxResult {
    let inserted = names.insert(String::from(canister_name));

    if !inserted {
        return if path.contains(&String::from(canister_name)) {
            path.push(String::from(canister_name));
            Err(DfxError::new(BuildError::DependencyError(format!(
                "Found circular dependency: {}",
                path.join(" -> ")
            ))))
        } else {
            Ok(())
        };
    }

    let canister_config = all_canisters
        .get(canister_name)
        .ok_or_else(|| anyhow!("Cannot find canister '{}'.", canister_name))?;

    path.push(String::from(canister_name));

    for canister in &canister_config.dependencies {
        add_dependencies(all_canisters, names, path, canister)?;
    }

    path.pop();

    Ok(())
}

#[derive(Clone)]
pub struct Config {
    path: PathBuf,
    json: Value,
    // public interface to the config:
    pub config: ConfigInterface,
}

#[allow(dead_code)]
impl Config {
    #[context("Failed to resolve config path from {}.", working_dir.to_string_lossy())]
    fn resolve_config_path(working_dir: &Path) -> DfxResult<Option<PathBuf>> {
        let mut curr = PathBuf::from(working_dir).canonicalize().with_context(|| {
            format!(
                "Failed to canonicalize working dir path {:}.",
                working_dir.to_string_lossy()
            )
        })?;
        while curr.parent().is_some() {
            if curr.join(CONFIG_FILE_NAME).is_file() {
                return Ok(Some(curr.join(CONFIG_FILE_NAME)));
            } else {
                curr.pop();
            }
        }

        // Have to check if the config could be in the root (e.g. on VMs / CI).
        if curr.join(CONFIG_FILE_NAME).is_file() {
            return Ok(Some(curr.join(CONFIG_FILE_NAME)));
        }

        Ok(None)
    }

    #[context("Failed to load config from {}.", path.to_string_lossy())]
    fn from_file(path: &Path) -> DfxResult<Config> {
        let content = std::fs::read(&path)
            .with_context(|| format!("Failed to read {}.", path.to_string_lossy()))?;
        Ok(Config::from_slice(path.to_path_buf(), &content)?)
    }

    #[context("Failed to read config from directory {}.", working_dir.to_string_lossy())]
    pub fn from_dir(working_dir: &Path) -> DfxResult<Option<Config>> {
        let path = Config::resolve_config_path(working_dir)?;
        let maybe_config = path.map(|path| Config::from_file(&path)).transpose()?;
        Ok(maybe_config)
    }

    #[context("Failed to read config from current working directory.")]
    pub fn from_current_dir() -> DfxResult<Option<Config>> {
        Config::from_dir(
            &std::env::current_dir().context("Failed to determine current working dir.")?,
        )
    }

    fn from_slice(path: PathBuf, content: &[u8]) -> std::io::Result<Config> {
        let config = serde_json::from_slice(content)?;
        let json = serde_json::from_slice(content)?;
        Ok(Config { path, json, config })
    }

    /// Create a configuration from a string.
    pub fn from_str(content: &str) -> std::io::Result<Config> {
        Config::from_slice(PathBuf::from("-"), content.as_bytes())
    }

    #[cfg(test)]
    pub fn from_str_and_path(path: PathBuf, content: &str) -> std::io::Result<Config> {
        Config::from_slice(path, content.as_bytes())
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_temp_path(&self) -> PathBuf {
        self.get_path().parent().unwrap().join(".dfx")
    }
    pub fn get_json(&self) -> &Value {
        &self.json
    }
    pub fn get_mut_json(&mut self) -> &mut Value {
        &mut self.json
    }
    pub fn get_config(&self) -> &ConfigInterface {
        &self.config
    }

    pub fn get_project_root(&self) -> &Path {
        // a configuration path contains a file name specifically. As
        // such we should be returning at least root as parent. If
        // this is invariance is broken, we must fail.
        self.path.parent().expect(
            "An incorrect configuration path was set with no parent, i.e. did not include root",
        )
    }

    pub fn save(&self) -> DfxResult {
        let json_pretty = serde_json::to_string_pretty(&self.json)
            .map_err(|e| error_invalid_data!("Failed to serialize dfx.json: {}", e))?;
        std::fs::write(&self.path, json_pretty).with_context(|| {
            format!("Failed to write config to {}.", self.path.to_string_lossy())
        })?;
        Ok(())
    }
}

// grumble grumble https://github.com/serde-rs/serde/issues/2231
impl<'de> Deserialize<'de> for CanisterTypeProperties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PropertiesVisitor)
    }
}

struct PropertiesVisitor;

impl<'de> Visitor<'de> for PropertiesVisitor {
    type Value = CanisterTypeProperties;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("canister type metadata")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let missing_field = A::Error::missing_field;
        let mut wasm: Option<String> = None;
        let mut candid: Option<String> = None;
        let (mut package, mut source, mut build, mut r#type) = (None, None, None, None);
        while let Some(key) = map.next_key::<String>()? {
            match &*key {
                "package" => package = Some(map.next_value()?),
                "source" => source = Some(map.next_value()?),
                "candid" => candid = Some(map.next_value()?),
                "build" => build = Some(map.next_value()?),
                "wasm" => wasm = Some(map.next_value()?),
                "type" => r#type = Some(map.next_value::<String>()?),
                _ => continue,
            }
        }
        let props = match r#type.as_deref() {
            Some("motoko") | None => CanisterTypeProperties::Motoko,
            Some("rust") => CanisterTypeProperties::Rust {
                candid: PathBuf::from(candid.ok_or_else(|| missing_field("candid"))?),
                package: package.ok_or_else(|| missing_field("package"))?,
            },
            Some("assets") => CanisterTypeProperties::Assets {
                source: source.ok_or_else(|| missing_field("source"))?,
            },
            Some("custom") => CanisterTypeProperties::Custom {
                build: build.unwrap_or_default(),
                candid: candid.ok_or_else(|| missing_field("candid"))?,
                wasm: wasm.ok_or_else(|| missing_field("wasm"))?,
            },
            Some(x) => {
                return Err(A::Error::unknown_variant(
                    x,
                    &["motoko", "rust", "assets", "custom"],
                ))
            }
        };
        Ok(props)
    }
}

#[derive(Clone)]
pub struct NetworksConfig {
    path: PathBuf,
    json: Value,
    // public interface to the networsk config:
    networks_config: NetworksConfigInterface,
}

impl NetworksConfig {
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_interface(&self) -> &NetworksConfigInterface {
        &self.networks_config
    }
    #[context("Failed to determine shared network data directory.")]
    pub fn get_network_data_directory(network: &str) -> DfxResult<PathBuf> {
        let project_dirs = ProjectDirs::from("org", "dfinity", "dfx").ok_or_else(|| {
            anyhow!("Unable to retrieve a valid home directory path from the operating system")
        })?;
        Ok(project_dirs.data_local_dir().join("network").join(network))
    }

    #[context("Failed to read shared networks configuration.")]
    pub fn new() -> DfxResult<NetworksConfig> {
        let dir = get_config_dfx_dir_path()?;

        let path = dir.join("networks.json");
        if path.exists() {
            NetworksConfig::from_file(&path)
        } else {
            Ok(NetworksConfig {
                path,
                json: Default::default(),
                networks_config: NetworksConfigInterface {
                    networks: BTreeMap::new(),
                },
            })
        }
    }

    #[context("Failed to read shared configuration from {}.", path.to_string_lossy())]
    fn from_file(path: &Path) -> DfxResult<NetworksConfig> {
        let content = std::fs::read(&path)
            .with_context(|| format!("Failed to read {}.", path.to_string_lossy()))?;

        let networks: BTreeMap<String, ConfigNetwork> = serde_json::from_slice(&content)?;
        let networks_config = NetworksConfigInterface { networks };
        let json = serde_json::from_slice(&content)?;
        let path = PathBuf::from(path);
        Ok(NetworksConfig {
            path,
            json,
            networks_config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_dfinity_config_current_path() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.into_path().canonicalize().unwrap();
        let config_path = root_path.join("foo/fah/bar").join(CONFIG_FILE_NAME);

        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "{}").unwrap();

        assert_eq!(
            config_path,
            Config::resolve_config_path(config_path.parent().unwrap())
                .unwrap()
                .unwrap(),
        );
    }

    #[test]
    fn find_dfinity_config_parent() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.into_path().canonicalize().unwrap();
        let config_path = root_path.join("foo/fah/bar").join(CONFIG_FILE_NAME);

        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "{}").unwrap();

        assert!(
            Config::resolve_config_path(config_path.parent().unwrap().parent().unwrap())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn find_dfinity_config_subdir() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.into_path().canonicalize().unwrap();
        let config_path = root_path.join("foo/fah/bar").join(CONFIG_FILE_NAME);
        let subdir_path = config_path.parent().unwrap().join("baz/blue");

        std::fs::create_dir_all(&subdir_path).unwrap();
        std::fs::write(&config_path, "{}").unwrap();

        assert_eq!(
            config_path,
            Config::resolve_config_path(subdir_path.as_path())
                .unwrap()
                .unwrap(),
        );
    }

    #[test]
    fn local_defaults_to_ephemeral() {
        let config = Config::from_str(
            r#"{
            "networks": {
                "local": {
                    "bind": "localhost:8000"
                }
            }
        }"#,
        )
        .unwrap();

        let network = config.get_config().get_network("local").unwrap();
        if let ConfigNetwork::ConfigLocalProvider(local_network) = network {
            assert_eq!(local_network.r#type, NetworkType::Ephemeral);
        } else {
            panic!("not a local provider");
        }
    }

    #[test]
    fn local_can_override_to_persistent() {
        let config = Config::from_str(
            r#"{
            "networks": {
                "local": {
                    "bind": "localhost:8000",
                    "type": "persistent"
                }
            }
        }"#,
        )
        .unwrap();

        let network = config.get_config().get_network("local").unwrap();
        if let ConfigNetwork::ConfigLocalProvider(local_network) = network {
            assert_eq!(local_network.r#type, NetworkType::Persistent);
        } else {
            panic!("not a local provider");
        }
    }

    #[test]
    fn network_defaults_to_persistent() {
        let config = Config::from_str(
            r#"{
            "networks": {
                "somewhere": {
                    "providers": [ "https://1.2.3.4:5000" ]
                }
            }
        }"#,
        )
        .unwrap();

        let network = config.get_config().get_network("somewhere").unwrap();
        if let ConfigNetwork::ConfigNetworkProvider(network_provider) = network {
            assert_eq!(network_provider.r#type, NetworkType::Persistent);
        } else {
            panic!("not a network provider");
        }
    }

    #[test]
    fn network_can_override_to_ephemeral() {
        let config = Config::from_str(
            r#"{
            "networks": {
                "staging": {
                    "providers": [ "https://1.2.3.4:5000" ],
                    "type": "ephemeral"
                }
            }
        }"#,
        )
        .unwrap();

        let network = config.get_config().get_network("staging").unwrap();
        if let ConfigNetwork::ConfigNetworkProvider(network_provider) = network {
            assert_eq!(network_provider.r#type, NetworkType::Ephemeral);
        } else {
            panic!("not a network provider");
        }

        assert_eq!(
            config.get_config().get_network("staging").unwrap(),
            &ConfigNetwork::ConfigNetworkProvider(ConfigNetworkProvider {
                providers: vec![String::from("https://1.2.3.4:5000")],
                r#type: NetworkType::Ephemeral,
            })
        );
    }

    #[test]
    fn get_correct_initialization_values() {
        let config = Config::from_str(
            r#"{
              "canisters": {
                "test_project": {
                  "initialization_values": {
                    "compute_allocation" : "100",
                    "memory_allocation": "8GB"
                  }
                }
              }
        }"#,
        )
        .unwrap();

        let config_interface = config.get_config();
        let compute_allocation = config_interface
            .get_compute_allocation("test_project")
            .unwrap()
            .unwrap();
        assert_eq!(100, compute_allocation);

        let memory_allocation = config_interface
            .get_memory_allocation("test_project")
            .unwrap()
            .unwrap();
        assert_eq!("8GB".parse::<Byte>().unwrap(), memory_allocation);

        let config_no_values = Config::from_str(
            r#"{
              "canisters": {
                "test_project_two": {
                }
              }
        }"#,
        )
        .unwrap();
        let config_interface = config_no_values.get_config();
        let compute_allocation = config_interface
            .get_compute_allocation("test_project_two")
            .unwrap();
        let memory_allocation = config_interface
            .get_memory_allocation("test_project_two")
            .unwrap();
        assert_eq!(None, compute_allocation);
        assert_eq!(None, memory_allocation);
    }
}
