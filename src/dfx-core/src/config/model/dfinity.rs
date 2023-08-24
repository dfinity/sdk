#![allow(dead_code)]
#![allow(clippy::should_implement_trait)] // for from_str.  why now?
use crate::config::directories::get_config_dfx_dir_path;
use crate::config::model::bitcoin_adapter::BitcoinAdapterLogLevel;
use crate::config::model::canister_http_adapter::HttpAdapterLogLevel;
use crate::error::dfx_config::DfxConfigError;
use crate::error::dfx_config::DfxConfigError::{
    CanisterCircularDependency, CanisterNotFound, CanistersFieldDoesNotExist,
    GetCanistersWithDependenciesFailed, GetComputeAllocationFailed, GetFreezingThresholdFailed,
    GetMemoryAllocationFailed, GetRemoteCanisterIdFailed, PullCanistersSameId,
};
use crate::error::load_dfx_config::LoadDfxConfigError;
use crate::error::load_dfx_config::LoadDfxConfigError::{
    DetermineCurrentWorkingDirFailed, LoadFromFileFailed, ResolveConfigPathFailed,
};
use crate::error::load_networks_config::LoadNetworksConfigError;
use crate::error::load_networks_config::LoadNetworksConfigError::{
    GetConfigPathFailed, LoadConfigFromFileFailed,
};
use crate::error::socket_addr_conversion::SocketAddrConversionError;
use crate::error::socket_addr_conversion::SocketAddrConversionError::{
    EmptyIterator, ParseSocketAddrFailed,
};
use crate::error::structured_file::StructuredFileError;
use crate::error::structured_file::StructuredFileError::{
    DeserializeJsonFileFailed, ReadJsonFileFailed,
};
use crate::json::save_json_file;
use crate::json::structure::{PossiblyStr, SerdeVec};
use byte_unit::Byte;
use candid::Principal;
use schemars::JsonSchema;
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::default::Default;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::network_descriptor::MOTOKO_PLAYGROUND_CANISTER_TIMEOUT_SECONDS;

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

/// # Wasm Optimization Levels
/// Wasm optimization levels that are passed to `wasm-opt`. "cycles" defaults to O3, "size" defaults to Oz.
/// O4 through O0 focus on performance (with O0 performing no optimizations), and Oz and Os focus on reducing binary size, where Oz is more aggressive than Os.
/// O3 and Oz empirically give best cycle savings and code size savings respectively.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WasmOptLevel {
    #[serde(rename = "cycles")]
    Cycles,
    #[serde(rename = "size")]
    Size,
    O4,
    O3,
    O2,
    O1,
    O0,
    Oz,
    Os,
}

impl std::fmt::Display for WasmOptLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum MetadataVisibility {
    /// Anyone can query the metadata
    #[default]
    Public,

    /// Only the controllers of the canister can query the metadata.
    Private,
}

/// # Canister Metadata Configuration
/// Configures a custom metadata section for the canister wasm.
/// dfx uses the first definition of a given name matching the current network, ignoring any of the same name that follow.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct CanisterMetadataSection {
    /// # Name
    /// The name of the wasm section
    pub name: String,

    /// # Visibility
    #[serde(default)]
    pub visibility: MetadataVisibility,

    /// # Networks
    /// Networks this section applies to.
    /// If this field is absent, then it applies to all networks.
    /// An empty array means this element will not apply to any network.
    pub networks: Option<BTreeSet<String>>,

    /// # Path
    /// Path to file containing section contents.
    /// Conflicts with `content`.
    /// For sections with name=`candid:service`, this field is optional, and if not specified, dfx will use
    /// the canister's candid definition.
    /// If specified for a Motoko canister, the service defined in the specified path must be a valid subtype of the canister's
    /// actual candid service definition.
    pub path: Option<PathBuf>,

    /// # Content
    /// Content of this metadata section.
    /// Conflicts with `path`.
    pub content: Option<String>,
}

impl CanisterMetadataSection {
    pub fn applies_to_network(&self, network: &str) -> bool {
        self.networks
            .as_ref()
            .map(|networks| networks.contains(network))
            .unwrap_or(true)
    }
}

/// # Pullable configuration
/// Configs the "pullable" metadata of the canister.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct PullableConfig {
    /// # wasm_url
    /// The Url to download canister wasm.
    /// Conflicts with `dynamic_wasm_url`.
    pub wasm_url: Option<String>,

    /// # Dynamic wasm_url
    /// Generate wasm_url dynamically.
    /// Conflicts with `wasm_url`.
    pub dynamic_wasm_url: Option<DynamicWasmUrl>,

    /// # wasm_hash
    /// SHA256 hash of the wasm module located at wasm_url.
    /// Only define this if the on-chain canister wasm is expected not to match the wasm at wasm_url.
    /// Conflicts with `wasm_hash_file` and `custom_wasm`.
    pub wasm_hash: Option<String>,

    /// # Path to wasm_hash
    /// Conflicts with `wasm_hash` and `custom_wasm`.
    pub wasm_hash_file: Option<String>,

    /// # Custom WASM
    /// Build a custom WASM for pullable.
    /// wasm_hash will be calculated from this custom WASM.
    /// Conflicts with `wasm_hash` and `wasm_hash_file`.
    pub custom_wasm: Option<CustomWasm>,

    /// # dependencies
    /// Canister IDs (Principal) of direct dependencies.
    #[schemars(with = "Vec::<String>")]
    pub dependencies: Vec<Principal>,

    /// # init_guide
    /// A message to guide consumers how to initialize the canister.
    pub init_guide: String,
}

/// # Dynamic wasm_url configuration
/// Configs how to generate wasm_url dynamically and where is the file contains wasm_url.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct DynamicWasmUrl {
    /// # Generate Commands
    /// Commands that are executed in order to generate this pullable canister's wasm_url dynamically.
    /// Expected to produce the wasm_url file in the path specified by the 'path' field.
    #[schemars(default)]
    pub generate: SerdeVec<String>,

    /// # wasm_url Path
    /// Path to the wasm_url file from the "generate" commands.
    /// The file should contains a valid URL.
    pub path: String,
}

/// # Custom WASM configuration
/// Configs how to generate a custom WASM for pullable and where is the custom WASM file.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct CustomWasm {
    /// # Generate Commands
    /// Commands that are executed in order to generate a custom wasm of this pullable canister.
    /// Expected to produce the wasm file in the path specified by the 'path' field.
    #[schemars(default)]
    pub generate: SerdeVec<String>,

    /// # Custom WASM Path
    /// Path to the custom wasm file from the "generate" commands.
    /// The file should be a valid canister WASM .
    pub path: String,
}

pub const DEFAULT_SHARED_LOCAL_BIND: &str = "127.0.0.1:4943"; // hex for "IC"
pub const DEFAULT_PROJECT_LOCAL_BIND: &str = "127.0.0.1:8000";
pub const DEFAULT_IC_GATEWAY: &str = "https://icp0.io";
pub const DEFAULT_IC_GATEWAY_TRAILING_SLASH: &str = "https://icp0.io/";
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

    /// # Canister-Specific Build Argument
    /// This field defines an additional argument to pass to the Motoko compiler when building the canister.
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
    /// Enabled by default for Rust/Motoko canisters.
    /// Disabled by default for custom canisters.
    pub shrink: Option<bool>,

    /// # Optimize Canister WASM
    /// Invoke wasm level optimizations after building the canister. Optimization level can be set to "cycles" to optimize for cycle usage, "size" to optimize for binary size, or any of "O4, O3, O2, O1, O0, Oz, Os".
    /// Disabled by default.
    /// If this option is specified, the `shrink` option will be ignored.
    #[serde(default)]
    pub optimize: Option<WasmOptLevel>,

    /// # Metadata
    /// Defines metadata sections to set in the canister .wasm
    #[serde(default)]
    pub metadata: Vec<CanisterMetadataSection>,

    /// # Pullable
    /// Defines required properties so that this canister is ready for `dfx deps pull` by other projects.
    #[serde(default)]
    pub pullable: Option<PullableConfig>,

    /// # Gzip Canister WASM
    /// Disabled by default.
    pub gzip: Option<bool>,
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

        /// # Build Commands
        /// Commands that are executed in order to produce this canister's assets.
        /// Expected to produce assets in one of the paths specified by the 'source' field.
        /// Optional if there is no build necessary or the assets can be built using the default `npm run build` command.
        #[schemars(default)]
        build: SerdeVec<String>,
    },
    /// # Custom-Specific Properties
    Custom {
        /// # WASM Path
        /// Path to WASM to be installed. URLs to a WASM module are also acceptable.
        /// A canister that has a URL to a WASM module can not also have `build` steps.
        wasm: String,

        /// # Candid File
        /// Path to this canister's candid interface declaration.  A URL to a candid file is also acceptable.
        candid: String,

        /// # Build Commands
        /// Commands that are executed in order to produce this canister's WASM module.
        /// Expected to produce the WASM in the path specified by the 'wasm' field.
        /// No build commands are allowed if the `wasm` field is a URL.
        #[schemars(default)]
        build: SerdeVec<String>,
    },
    /// # Motoko-Specific Properties
    Motoko,
    /// # Pull-Specific Properties
    Pull {
        /// # Canister ID
        /// Principal of the canister on the ic network.
        #[schemars(with = "String")]
        id: Principal,
    },
}

impl CanisterTypeProperties {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust { .. } => "rust",
            Self::Motoko { .. } => "motoko",
            Self::Assets { .. } => "assets",
            Self::Custom { .. } => "custom",
            Self::Pull { .. } => "pull",
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
    #[serde(default = "default_bitcoin_log_level")]
    pub log_level: BitcoinAdapterLogLevel,

    /// # Initialization Argument
    /// The initialization argument for the bitcoin canister.
    #[serde(default = "default_bitcoin_canister_init_arg")]
    pub canister_init_arg: String,
}

pub fn default_bitcoin_log_level() -> BitcoinAdapterLogLevel {
    BitcoinAdapterLogLevel::Info
}

pub fn default_bitcoin_canister_init_arg() -> String {
    "(record { stability_threshold = 0 : nat; network = variant { regtest }; blocks_source = principal \"aaaaa-aa\"; fees = record { get_utxos_base = 0 : nat; get_utxos_cycles_per_ten_instructions = 0 : nat; get_utxos_maximum = 0 : nat; get_balance = 0 : nat; get_balance_maximum = 0 : nat; get_current_fee_percentiles = 0 : nat; get_current_fee_percentiles_maximum = 0 : nat;  send_transaction_base = 0 : nat; send_transaction_per_byte = 0 : nat; }; syncing = variant { enabled }; api_access = variant { enabled }; disable_api_if_not_fully_synced = variant { enabled }})".to_string()
}

impl Default for ConfigDefaultsBitcoin {
    fn default() -> Self {
        ConfigDefaultsBitcoin {
            enabled: false,
            nodes: None,
            log_level: default_bitcoin_log_level(),
            canister_init_arg: default_bitcoin_canister_init_arg(),
        }
    }
}

/// # HTTP Adapter Configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    // We store ephemeral canister ids in .dfx/{network}/canister_ids.json
    #[default]
    Ephemeral,

    // We store persistent canister ids in canister_ids.json (adjacent to dfx.json)
    Persistent,
}

impl NetworkType {
    fn ephemeral() -> Self {
        NetworkType::Ephemeral
    }
    fn persistent() -> Self {
        NetworkType::Persistent
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReplicaSubnetType {
    System,
    #[default]
    Application,
    VerifiedApplication,
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

fn default_playground_timeout_seconds() -> u64 {
    MOTOKO_PLAYGROUND_CANISTER_TIMEOUT_SECONDS
}

/// Playground config to borrow canister from instead of creating new canisters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlaygroundConfig {
    /// Canister ID of the playground canister
    pub playground_canister: String,

    /// How many seconds a canister can be borrowed for
    #[serde(default = "default_playground_timeout_seconds")]
    pub timeout_seconds: u64,
}

/// # Custom Network Configuration
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigNetworkProvider {
    /// The URL(s) this network can be reached at.
    pub providers: Vec<String>,

    /// Persistence type of this network.
    #[serde(default = "NetworkType::persistent")]
    pub r#type: NetworkType,
    pub playground: Option<PlaygroundConfig>,
}

/// # Local Replica Configuration
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    pub playground: Option<PlaygroundConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
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

    /// If set, environment variables will be output to this file (without overwriting any user-defined variables, if the file already exists).
    pub output_env_file: Option<PathBuf>,
}

pub type TopLevelConfigNetworks = BTreeMap<String, ConfigNetwork>;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NetworksConfigInterface {
    pub networks: TopLevelConfigNetworks,
}

impl ConfigCanistersCanister {}

pub fn to_socket_addr(s: &str) -> Result<SocketAddr, SocketAddrConversionError> {
    match s.to_socket_addrs() {
        Ok(mut a) => match a.next() {
            Some(res) => Ok(res),
            None => Err(EmptyIterator(s.to_string())),
        },
        Err(err) => Err(ParseSocketAddrFailed(s.to_string(), err)),
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
    pub fn get_canister_names_with_dependencies(
        &self,
        some_canister: Option<&str>,
    ) -> Result<Vec<String>, DfxConfigError> {
        self.canisters
            .as_ref()
            .ok_or(CanistersFieldDoesNotExist())
            .and_then(|canister_map| match some_canister {
                Some(specific_canister) => {
                    let mut names = HashSet::new();
                    let mut path = vec![];
                    add_dependencies(canister_map, &mut names, &mut path, specific_canister)
                        .map(|_| names.into_iter().collect())
                }
                None => Ok(canister_map.keys().cloned().collect()),
            })
            .map_err(|cause| {
                GetCanistersWithDependenciesFailed(some_canister.map(String::from), Box::new(cause))
            })
    }

    pub fn get_remote_canister_id(
        &self,
        canister: &str,
        network: &str,
    ) -> Result<Option<Principal>, DfxConfigError> {
        let maybe_principal = self
            .get_canister_config(canister)
            .map_err(|e| {
                GetRemoteCanisterIdFailed(
                    Box::new(canister.to_string()),
                    Box::new(network.to_string()),
                    Box::new(e),
                )
            })?
            .remote
            .as_ref()
            .and_then(|r| r.id.get(network))
            .copied();

        Ok(maybe_principal)
    }

    pub fn is_remote_canister(
        &self,
        canister: &str,
        network: &str,
    ) -> Result<bool, DfxConfigError> {
        Ok(self.get_remote_canister_id(canister, network)?.is_some())
    }

    pub fn get_compute_allocation(
        &self,
        canister_name: &str,
    ) -> Result<Option<u64>, DfxConfigError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetComputeAllocationFailed(canister_name.to_string(), Box::new(e)))?
            .initialization_values
            .compute_allocation
            .map(|x| x.0))
    }

    pub fn get_memory_allocation(
        &self,
        canister_name: &str,
    ) -> Result<Option<Byte>, DfxConfigError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetMemoryAllocationFailed(canister_name.to_string(), Box::new(e)))?
            .initialization_values
            .memory_allocation)
    }

    pub fn get_freezing_threshold(
        &self,
        canister_name: &str,
    ) -> Result<Option<Duration>, DfxConfigError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetFreezingThresholdFailed(canister_name.to_string(), Box::new(e)))?
            .initialization_values
            .freezing_threshold)
    }

    fn get_canister_config(
        &self,
        canister_name: &str,
    ) -> Result<&ConfigCanistersCanister, DfxConfigError> {
        self.canisters
            .as_ref()
            .ok_or(CanistersFieldDoesNotExist())?
            .get(canister_name)
            .ok_or_else(|| CanisterNotFound(canister_name.to_string()))
    }

    pub fn get_pull_canisters(&self) -> Result<BTreeMap<String, Principal>, DfxConfigError> {
        let mut res = BTreeMap::new();
        let mut id_to_name: BTreeMap<Principal, &String> = BTreeMap::new();
        if let Some(map) = &self.canisters {
            for (k, v) in map {
                if let CanisterTypeProperties::Pull { id } = v.type_specific {
                    if let Some(other_name) = id_to_name.get(&id) {
                        return Err(PullCanistersSameId(other_name.to_string(), k.clone(), id));
                    }
                    res.insert(k.clone(), id);
                    id_to_name.insert(id, k);
                }
            }
        };
        Ok(res)
    }
}

fn add_dependencies(
    all_canisters: &BTreeMap<String, ConfigCanistersCanister>,
    names: &mut HashSet<String>,
    path: &mut Vec<String>,
    canister_name: &str,
) -> Result<(), DfxConfigError> {
    let inserted = names.insert(String::from(canister_name));

    if !inserted {
        return if path.contains(&String::from(canister_name)) {
            path.push(String::from(canister_name));
            Err(CanisterCircularDependency(path.clone()))
        } else {
            Ok(())
        };
    }

    let canister_config = all_canisters
        .get(canister_name)
        .ok_or_else(|| CanisterNotFound(canister_name.to_string()))?;

    path.push(String::from(canister_name));

    for canister in &canister_config.dependencies {
        add_dependencies(all_canisters, names, path, canister)?;
    }

    path.pop();

    Ok(())
}

#[derive(Clone, Debug)]
pub struct Config {
    path: PathBuf,
    json: Value,
    // public interface to the config:
    pub config: ConfigInterface,
}

#[allow(dead_code)]
impl Config {
    fn resolve_config_path(working_dir: &Path) -> Result<Option<PathBuf>, LoadDfxConfigError> {
        let mut curr = crate::fs::canonicalize(working_dir).map_err(ResolveConfigPathFailed)?;
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

    fn from_file(path: &Path) -> Result<Config, StructuredFileError> {
        let content = crate::fs::read(path).map_err(ReadJsonFileFailed)?;
        Config::from_slice(path.to_path_buf(), &content)
    }

    pub fn from_dir(working_dir: &Path) -> Result<Option<Config>, LoadDfxConfigError> {
        let path = Config::resolve_config_path(working_dir)?;
        path.map(|path| Config::from_file(&path))
            .transpose()
            .map_err(LoadFromFileFailed)
    }

    pub fn from_current_dir() -> Result<Option<Config>, LoadDfxConfigError> {
        Config::from_dir(&std::env::current_dir().map_err(DetermineCurrentWorkingDirFailed)?)
    }

    fn from_slice(path: PathBuf, content: &[u8]) -> Result<Config, StructuredFileError> {
        let config = serde_json::from_slice(content)
            .map_err(|e| DeserializeJsonFileFailed(Box::new(path.clone()), e))?;
        let json = serde_json::from_slice(content)
            .map_err(|e| DeserializeJsonFileFailed(Box::new(path.clone()), e))?;
        Ok(Config { path, json, config })
    }

    /// Create a configuration from a string.
    #[cfg(test)]
    pub(crate) fn from_str(content: &str) -> Result<Config, StructuredFileError> {
        Config::from_slice(PathBuf::from("-"), content.as_bytes())
    }

    #[cfg(test)]
    pub(crate) fn from_str_and_path(
        path: PathBuf,
        content: &str,
    ) -> Result<Config, StructuredFileError> {
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

    pub fn save(&self) -> Result<(), StructuredFileError> {
        save_json_file(&self.path, &self.json)
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
        let mut wasm = None;
        let mut candid = None;
        let mut package = None;
        let mut source = None;
        let mut build = None;
        let mut r#type = None;
        let mut id = None;
        while let Some(key) = map.next_key::<String>()? {
            match &*key {
                "package" => package = Some(map.next_value()?),
                "source" => source = Some(map.next_value()?),
                "candid" => candid = Some(map.next_value()?),
                "build" => build = Some(map.next_value()?),
                "wasm" => wasm = Some(map.next_value()?),
                "type" => r#type = Some(map.next_value::<String>()?),
                "id" => id = Some(map.next_value()?),
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
                build: build.unwrap_or_default(),
            },
            Some("custom") => CanisterTypeProperties::Custom {
                build: build.unwrap_or_default(),
                candid: candid.ok_or_else(|| missing_field("candid"))?,
                wasm: wasm.ok_or_else(|| missing_field("wasm"))?,
            },
            Some("pull") => CanisterTypeProperties::Pull {
                id: id.ok_or_else(|| missing_field("id"))?,
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

    pub fn new() -> Result<NetworksConfig, LoadNetworksConfigError> {
        let dir = get_config_dfx_dir_path().map_err(GetConfigPathFailed)?;

        let path = dir.join("networks.json");
        if path.exists() {
            NetworksConfig::from_file(&path).map_err(LoadConfigFromFileFailed)
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

    fn from_file(path: &Path) -> Result<NetworksConfig, StructuredFileError> {
        let content = crate::fs::read(path).map_err(ReadJsonFileFailed)?;

        let networks: BTreeMap<String, ConfigNetwork> = serde_json::from_slice(&content)
            .map_err(|e| DeserializeJsonFileFailed(Box::new(path.to_path_buf()), e))?;
        let networks_config = NetworksConfigInterface { networks };
        let json = serde_json::from_slice(&content)
            .map_err(|e| DeserializeJsonFileFailed(Box::new(path.to_path_buf()), e))?;
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
                playground: None,
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
