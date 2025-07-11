#![allow(dead_code)]
#![allow(clippy::should_implement_trait)] // for from_str.  why now?
use crate::config::directories::get_user_dfx_config_dir;
use crate::config::model::bitcoin_adapter::BitcoinAdapterLogLevel;
use crate::config::model::canister_http_adapter::HttpAdapterLogLevel;
use crate::config::model::extension_canister_type::apply_extension_canister_types;
use crate::error::config::{GetOutputEnvFileError, GetTempPathError};
use crate::error::dfx_config::AddDependenciesError::CanisterCircularDependency;
use crate::error::dfx_config::GetCanisterNamesWithDependenciesError::AddDependenciesFailed;
use crate::error::dfx_config::GetComputeAllocationError::GetComputeAllocationFailed;
use crate::error::dfx_config::GetFreezingThresholdError::GetFreezingThresholdFailed;
use crate::error::dfx_config::GetLogVisibilityError::GetLogVisibilityFailed;
use crate::error::dfx_config::GetMemoryAllocationError::GetMemoryAllocationFailed;
use crate::error::dfx_config::GetPullCanistersError::PullCanistersSameId;
use crate::error::dfx_config::GetRemoteCanisterIdError::GetRemoteCanisterIdFailed;
use crate::error::dfx_config::GetReservedCyclesLimitError::GetReservedCyclesLimitFailed;
use crate::error::dfx_config::GetSpecifiedIdError::GetSpecifiedIdFailed;
use crate::error::dfx_config::GetWasmMemoryLimitError::GetWasmMemoryLimitFailed;
use crate::error::dfx_config::GetWasmMemoryThresholdError::GetWasmMemoryThresholdFailed;
use crate::error::dfx_config::{
    AddDependenciesError, GetCanisterConfigError, GetCanisterNamesWithDependenciesError,
    GetComputeAllocationError, GetFreezingThresholdError, GetLogVisibilityError,
    GetMemoryAllocationError, GetPullCanistersError, GetRemoteCanisterIdError,
    GetReservedCyclesLimitError, GetSpecifiedIdError, GetWasmMemoryLimitError,
    GetWasmMemoryThresholdError,
};
use crate::error::fs::CanonicalizePathError;
use crate::error::load_dfx_config::LoadDfxConfigError;
use crate::error::load_dfx_config::LoadDfxConfigError::{
    DetermineCurrentWorkingDirFailed, ResolveConfigPath,
};
use crate::error::load_networks_config::LoadNetworksConfigError;
use crate::error::load_networks_config::LoadNetworksConfigError::{
    GetConfigPathFailed as GetNetworkConfigPathFailed,
    LoadConfigFromFileFailed as LoadNetworkConfigFromFileFailed,
};
use crate::error::load_tool_config::ToolConfigError;
use crate::error::load_tool_config::ToolConfigError::{
    GetConfigPathFailed as GetToolConfigPathFailed,
    LoadConfigFromFileFailed as LoadToolConfigFromFileFailed, SaveDefaultConfigFailed,
};
use crate::error::socket_addr_conversion::SocketAddrConversionError;
use crate::error::socket_addr_conversion::SocketAddrConversionError::{
    EmptyIterator, ParseSocketAddrFailed,
};
use crate::error::structured_file::StructuredFileError;
use crate::error::structured_file::StructuredFileError::DeserializeJsonFileFailed;
use crate::extension::manager::ExtensionManager;
use crate::fs::create_dir_all;
use crate::json::structure::{PossiblyStr, SerdeVec};
use crate::json::{load_json_file, save_json_file};
use crate::util::ByteSchema;
use byte_unit::Byte;
use candid::Principal;
use clap::ValueEnum;
use ic_utils::interfaces::management_canister::LogVisibility;
use schemars::JsonSchema;
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::network_descriptor::MOTOKO_PLAYGROUND_CANISTER_TIMEOUT_SECONDS;

pub const CONFIG_FILE_NAME: &str = "dfx.json";

pub const BUILTIN_CANISTER_TYPES: [&str; 5] = ["rust", "motoko", "assets", "custom", "pull"];

const EMPTY_CONFIG_DEFAULTS: ConfigDefaults = ConfigDefaults {
    bitcoin: None,
    bootstrap: None,
    build: None,
    canister_http: None,
    proxy: None,
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
impl Display for WasmOptLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Pullable {
    /// # wasm_url
    /// The Url to download canister wasm.
    pub wasm_url: String,
    /// # wasm_hash
    /// SHA256 hash of the wasm module located at wasm_url.
    /// Only define this if the on-chain canister wasm is expected not to match the wasm at wasm_url.
    /// The hash can also be specified via a URL using the `wasm_hash_url` field.
    /// If both are defined, the `wasm_hash_url` field will be ignored.
    pub wasm_hash: Option<String>,
    /// # wasm_hash_url
    /// Specify the SHA256 hash of the wasm module via this URL.
    /// Only define this if the on-chain canister wasm is expected not to match the wasm at wasm_url.
    /// The hash can also be specified directly using the `wasm_hash` field.
    /// If both are defined, the `wasm_hash_url` field will be ignored.
    pub wasm_hash_url: Option<String>,
    /// # dependencies
    /// Canister IDs (Principal) of direct dependencies.
    #[schemars(with = "Vec::<String>")]
    pub dependencies: Vec<Principal>,
    /// # init_guide
    /// A message to guide consumers how to initialize the canister.
    pub init_guide: String,
    /// # init_arg
    /// A default initialization argument for the canister that consumers can use.
    pub init_arg: Option<String>,
}

pub type TechStackCategoryMap = HashMap<String, HashMap<String, String>>;

/// # Tech Stack
/// The tech stack used to build a canister.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct TechStack {
    /// # cdk
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdk: Option<TechStackCategoryMap>,
    /// # language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<TechStackCategoryMap>,
    /// # lib
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<TechStackCategoryMap>,
    /// # tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<TechStackCategoryMap>,
    /// # other
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<TechStackCategoryMap>,
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

    /// # Pre-Install Commands
    /// One or more commands to run pre canister installation.
    /// These commands are executed in the root of the project.
    #[serde(default)]
    pub pre_install: SerdeVec<String>,

    /// # Post-Install Commands
    /// One or more commands to run post canister installation.
    /// These commands are executed in the root of the project.
    #[serde(default)]
    pub post_install: SerdeVec<String>,

    /// # Path to Canister Entry Point
    /// Entry point for e.g. Motoko Compiler.
    pub main: Option<PathBuf>,

    /// # Shrink Canister Wasm
    /// Whether run `ic-wasm shrink` after building the Canister.
    /// Enabled by default for Rust/Motoko canisters.
    /// Disabled by default for custom canisters.
    pub shrink: Option<bool>,

    /// # Optimize Canister Wasm
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
    pub pullable: Option<Pullable>,

    /// # Tech Stack
    /// Defines the tech stack used to build this canister.
    #[serde(default)]
    pub tech_stack: Option<TechStack>,

    /// # Gzip Canister Wasm
    /// Disabled by default.
    pub gzip: Option<bool>,

    /// # Specified Canister ID
    /// Attempts to create the canister with this Canister ID.
    /// This option only works with non-mainnet replica.
    /// If the `--specified-id` argument is also provided, this `specified_id` field will be ignored.
    #[schemars(with = "Option<String>")]
    pub specified_id: Option<Principal>,

    /// # Init Arg
    /// The Candid initialization argument for installing the canister.
    /// If the `--argument` or `--argument-file` argument is also provided, this `init_arg` field will be ignored.
    pub init_arg: Option<String>,

    /// # Init Arg File
    /// The Candid initialization argument file for installing the canister.
    /// If the `--argument` or `--argument-file` argument is also provided, this `init_arg_file` field will be ignored.
    pub init_arg_file: Option<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CanisterTypeProperties {
    /// # Rust-Specific Properties
    Rust {
        /// # Package Name
        /// Name of the Rust package that compiles this canister's Wasm.
        package: String,

        /// # Crate name
        /// Name of the Rust crate that compiles to this canister's Wasm.
        /// If left unspecified, defaults to the crate with the same name as the package.
        #[serde(rename = "crate")]
        crate_name: Option<String>,

        /// # Candid File
        /// Path of this canister's candid interface declaration.
        candid: PathBuf,

        /// # `cargo-audit` check
        /// If set to true, does not run `cargo audit` before building.
        #[serde(default)]
        skip_cargo_audit: bool,
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

        /// # NPM workspace
        /// The workspace in package.json that this canister is in, if it is not in the root workspace.
        workspace: Option<String>,
    },
    /// # Custom-Specific Properties
    Custom {
        /// # Wasm Path
        /// Path to Wasm to be installed. URLs to a Wasm module are also acceptable.
        /// A canister that has a URL to a Wasm module can not also have `build` steps.
        wasm: String,

        /// # Candid File
        /// Path to this canister's candid interface declaration.  A URL to a candid file is also acceptable.
        candid: String,

        /// # Build Commands
        /// Commands that are executed in order to produce this canister's Wasm module.
        /// Expected to produce the Wasm in the path specified by the 'wasm' field.
        /// No build commands are allowed if the `wasm` field is a URL.
        /// These commands are executed in the root of the project.
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

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CanisterLogVisibility {
    #[default]
    Controllers,
    Public,
    #[schemars(with = "Vec::<String>")]
    AllowedViewers(Vec<Principal>),
}

impl From<CanisterLogVisibility> for LogVisibility {
    fn from(value: CanisterLogVisibility) -> Self {
        match value {
            CanisterLogVisibility::Controllers => LogVisibility::Controllers,
            CanisterLogVisibility::Public => LogVisibility::Public,
            CanisterLogVisibility::AllowedViewers(viewers) => {
                LogVisibility::AllowedViewers(viewers)
            }
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
    /// Can be specified as an integer, or as an SI unit string (e.g. "4KB", "2 MiB")
    #[schemars(with = "Option<ByteSchema>")]
    pub memory_allocation: Option<Byte>,

    /// # Freezing Threshold
    /// Freezing threshould of the canister, measured in seconds.
    /// Valid inputs are numbers (seconds) or strings parsable by humantime (e.g. "15days 2min 2s").
    #[serde(with = "humantime_serde")]
    #[schemars(with = "Option<String>")]
    pub freezing_threshold: Option<Duration>,

    /// # Reserved Cycles Limit
    /// Specifies the upper limit of the canister's reserved cycles balance.
    ///
    /// Reserved cycles are cycles that the system sets aside for future use by the canister.
    /// If a subnet's storage exceeds 450 GiB, then every time a canister allocates new storage bytes,
    /// the system sets aside some amount of cycles from the main balance of the canister.
    /// These reserved cycles will be used to cover future payments for the newly allocated bytes.
    /// The reserved cycles are not transferable and the amount of reserved cycles depends on how full the subnet is.
    ///
    /// A setting of 0 means that the canister will trap if it tries to allocate new storage while the subnet's memory usage exceeds 450 GiB.
    #[schemars(with = "Option<u128>")]
    pub reserved_cycles_limit: Option<u128>,

    /// # Wasm Memory Limit
    /// Specifies a soft limit (in bytes) on the Wasm memory usage of the canister.
    ///
    /// Update calls, timers, heartbeats, installs, and post-upgrades fail if the
    /// Wasm memory usage exceeds this limit. The main purpose of this setting is
    /// to protect against the case when the canister reaches the hard 4GiB
    /// limit.
    ///
    /// Must be a number of bytes between 0 and 2^48 (i.e. 256 TiB), inclusive.
    /// Can be specified as an integer, or as an SI unit string (e.g. "4KB", "2 MiB")
    #[schemars(with = "Option<ByteSchema>")]
    pub wasm_memory_limit: Option<Byte>,

    /// # Wasm Memory Threshold
    ///
    /// Specifies a threshold (in bytes) on the Wasm memory usage of the canister,
    /// as a distance from `wasm_memory_limit`.
    ///
    /// When the remaining memory before the limit drops below this threshold, its
    /// `on_low_wasm_memory` hook will be invoked. This enables it to self-optimize,
    /// or raise an alert, or otherwise attempt to prevent itself from reaching
    /// `wasm_memory_limit`.
    ///
    /// Must be a number of bytes between 0 and 2^48 (i.e. 256 TiB), inclusive.
    /// Can be specified as an integer, or as an SI unit string (e.g. "4KB", "2 MiB")
    #[schemars(with = "Option<ByteSchema>")]
    pub wasm_memory_threshold: Option<Byte>,

    /// # Log Visibility
    /// Specifies who is allowed to read the canister's logs.
    ///
    /// Can be "public", "controllers" or "allowed_viewers" with a list of principals.
    #[schemars(with = "Option<CanisterLogVisibility>")]
    pub log_visibility: Option<CanisterLogVisibility>,
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
    /// A string that will replace process.env.CANISTER_ID_{canister_name_uppercase}
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
    "(record { stability_threshold = 0 : nat; network = variant { regtest }; blocks_source = principal \"aaaaa-aa\"; fees = record { get_utxos_base = 50000000 : nat; get_utxos_cycles_per_ten_instructions = 10 : nat; get_utxos_maximum = 10000000000 : nat; get_balance = 10000000 : nat; get_balance_maximum = 100000000 : nat; get_block_headers_base = 50000000 : nat; get_block_headers_cycles_per_ten_instructions = 10 : nat; get_block_headers_maximum = 10000000000 : nat; get_current_fee_percentiles = 10000000 : nat; get_current_fee_percentiles_maximum = 100000000 : nat; send_transaction_base = 5000000000 : nat; send_transaction_per_byte = 20000000 : nat; }; syncing = variant { enabled }; api_access = variant { enabled }; disable_api_if_not_fully_synced = variant { enabled }})".to_string()
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
/// The bootstrap command has been removed.  All of these fields are ignored.
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
    /// This command is executed in the root of the project.
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
    pub fn to_ic_starter_string(&self) -> String {
        match self {
            Self::Critical => "critical".to_string(),
            Self::Error => "error".to_string(),
            Self::Warning => "warning".to_string(),
            Self::Info => "info".to_string(),
            Self::Debug => "debug".to_string(),
            Self::Trace => "trace".to_string(),
        }
    }
    pub fn to_pocketic_string(&self) -> String {
        match self {
            Self::Critical => "CRITICAL".to_string(),
            Self::Error => "ERROR".to_string(),
            Self::Warning => "WARN".to_string(),
            Self::Info => "INFO".to_string(),
            Self::Debug => "DEBUG".to_string(),
            Self::Trace => "TRACE".to_string(),
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

/// Configuration for the HTTP gateway.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDefaultsProxy {
    /// A list of domains that can be served. These are used for canister resolution [default: localhost]
    pub domain: Option<SerdeVec<String>>,
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
    pub proxy: Option<ConfigDefaultsProxy>,
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
    pub proxy: Option<ConfigDefaultsProxy>,
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
    ) -> Result<Vec<String>, GetCanisterNamesWithDependenciesError> {
        self.canisters
            .as_ref()
            .ok_or(GetCanisterNamesWithDependenciesError::CanistersFieldDoesNotExist())
            .and_then(|canister_map| match some_canister {
                Some(specific_canister) => {
                    let mut names = HashSet::new();
                    let mut path = vec![];
                    add_dependencies(canister_map, &mut names, &mut path, specific_canister)
                        .map(|_| names.into_iter().collect())
                        .map_err(|err| AddDependenciesFailed(specific_canister.to_string(), err))
                }
                None => Ok(canister_map.keys().cloned().collect()),
            })
    }

    pub fn get_remote_canister_id(
        &self,
        canister: &str,
        network: &str,
    ) -> Result<Option<Principal>, GetRemoteCanisterIdError> {
        let maybe_principal = self
            .get_canister_config(canister)
            .map_err(|e| {
                GetRemoteCanisterIdFailed(
                    Box::new(canister.to_string()),
                    Box::new(network.to_string()),
                    e,
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
    ) -> Result<bool, GetRemoteCanisterIdError> {
        Ok(self.get_remote_canister_id(canister, network)?.is_some())
    }

    pub fn get_compute_allocation(
        &self,
        canister_name: &str,
    ) -> Result<Option<u64>, GetComputeAllocationError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetComputeAllocationFailed(canister_name.to_string(), e))?
            .initialization_values
            .compute_allocation
            .map(|x| x.0))
    }

    pub fn get_memory_allocation(
        &self,
        canister_name: &str,
    ) -> Result<Option<Byte>, GetMemoryAllocationError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetMemoryAllocationFailed(canister_name.to_string(), e))?
            .initialization_values
            .memory_allocation)
    }

    pub fn get_freezing_threshold(
        &self,
        canister_name: &str,
    ) -> Result<Option<Duration>, GetFreezingThresholdError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetFreezingThresholdFailed(canister_name.to_string(), e))?
            .initialization_values
            .freezing_threshold)
    }

    pub fn get_reserved_cycles_limit(
        &self,
        canister_name: &str,
    ) -> Result<Option<u128>, GetReservedCyclesLimitError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetReservedCyclesLimitFailed(canister_name.to_string(), e))?
            .initialization_values
            .reserved_cycles_limit)
    }

    pub fn get_wasm_memory_limit(
        &self,
        canister_name: &str,
    ) -> Result<Option<Byte>, GetWasmMemoryLimitError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetWasmMemoryLimitFailed(canister_name.to_string(), e))?
            .initialization_values
            .wasm_memory_limit)
    }

    pub fn get_wasm_memory_threshold(
        &self,
        canister_name: &str,
    ) -> Result<Option<Byte>, GetWasmMemoryThresholdError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetWasmMemoryThresholdFailed(canister_name.to_string(), e))?
            .initialization_values
            .wasm_memory_threshold)
    }

    pub fn get_log_visibility(
        &self,
        canister_name: &str,
    ) -> Result<Option<LogVisibility>, GetLogVisibilityError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetLogVisibilityFailed(canister_name.to_string(), e))?
            .initialization_values
            .log_visibility
            .clone()
            .map(|visibility| visibility.into()))
    }

    fn get_canister_config(
        &self,
        canister_name: &str,
    ) -> Result<&ConfigCanistersCanister, GetCanisterConfigError> {
        self.canisters
            .as_ref()
            .ok_or(GetCanisterConfigError::CanistersFieldDoesNotExist())?
            .get(canister_name)
            .ok_or_else(|| GetCanisterConfigError::CanisterNotFound(canister_name.to_string()))
    }

    pub fn get_pull_canisters(&self) -> Result<BTreeMap<String, Principal>, GetPullCanistersError> {
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

    pub fn get_specified_id(
        &self,
        canister_name: &str,
    ) -> Result<Option<Principal>, GetSpecifiedIdError> {
        Ok(self
            .get_canister_config(canister_name)
            .map_err(|e| GetSpecifiedIdFailed(canister_name.to_string(), e))?
            .specified_id)
    }
}

fn add_dependencies(
    all_canisters: &BTreeMap<String, ConfigCanistersCanister>,
    names: &mut HashSet<String>,
    path: &mut Vec<String>,
    canister_name: &str,
) -> Result<(), AddDependenciesError> {
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
        .ok_or_else(|| AddDependenciesError::CanisterNotFound(canister_name.to_string()))?;

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
    fn resolve_config_path(working_dir: &Path) -> Result<Option<PathBuf>, CanonicalizePathError> {
        let mut curr = crate::fs::canonicalize(working_dir)?;
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

    fn from_file(
        path: &Path,
        extension_manager: Option<&ExtensionManager>,
    ) -> Result<Config, LoadDfxConfigError> {
        let content = crate::fs::read(path)?;
        Config::from_slice(path.to_path_buf(), &content, extension_manager)
    }

    pub fn from_dir(
        working_dir: &Path,
        extension_manager: Option<&ExtensionManager>,
    ) -> Result<Option<Config>, LoadDfxConfigError> {
        let path = Config::resolve_config_path(working_dir).map_err(ResolveConfigPath)?;
        path.map(|path| Config::from_file(&path, extension_manager))
            .transpose()
    }

    pub fn from_current_dir(
        extension_manager: Option<&ExtensionManager>,
    ) -> Result<Option<Config>, LoadDfxConfigError> {
        let working_dir = std::env::current_dir().map_err(DetermineCurrentWorkingDirFailed)?;
        Config::from_dir(&working_dir, extension_manager)
    }

    fn from_slice(
        path: PathBuf,
        content: &[u8],
        extension_manager: Option<&ExtensionManager>,
    ) -> Result<Config, LoadDfxConfigError> {
        let json: Value = serde_json::from_slice(content)
            .map_err(|e| LoadDfxConfigError::DeserializeValueFailed(Box::new(path.clone()), e))?;
        let effective_json = apply_extension_canister_types(json.clone(), extension_manager)?;

        let config = serde_json::from_value(effective_json)
            .map_err(|e| LoadDfxConfigError::DeserializeValueFailed(Box::new(path.clone()), e))?;
        Ok(Config { path, json, config })
    }

    /// Create a configuration from a string.
    #[cfg(test)]
    pub(crate) fn from_str(content: &str) -> Result<Config, StructuredFileError> {
        Ok(Config::from_slice(PathBuf::from("-"), content.as_bytes(), None).unwrap())
    }

    #[cfg(test)]
    pub(crate) fn from_str_and_path(
        path: PathBuf,
        content: &str,
    ) -> Result<Config, StructuredFileError> {
        Ok(Config::from_slice(path, content.as_bytes(), None).unwrap())
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_temp_path(&self) -> Result<PathBuf, GetTempPathError> {
        let path = self.get_path().parent().unwrap().join(".dfx");
        create_dir_all(&path)?;
        Ok(path)
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

    // returns the path to the output env file if any, guaranteed to be
    // a child relative to the project root
    pub fn get_output_env_file(
        &self,
        from_cmdline: Option<PathBuf>,
    ) -> Result<Option<PathBuf>, GetOutputEnvFileError> {
        from_cmdline
            .or(self.config.output_env_file.clone())
            .map(|p| {
                if p.is_relative() {
                    let p = self.get_project_root().join(p);

                    // cannot canonicalize a path that doesn't exist, but the parent should exist
                    let env_parent = crate::fs::parent(&p)?;
                    let env_parent = crate::fs::canonicalize(&env_parent)?;
                    if !env_parent.starts_with(self.get_project_root()) {
                        Err(GetOutputEnvFileError::OutputEnvFileMustBeInProjectRoot(p))
                    } else {
                        Ok(self.get_project_root().join(p))
                    }
                } else {
                    Err(GetOutputEnvFileError::OutputEnvFileMustBeRelative(p))
                }
            })
            .transpose()
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
        let mut skip_cargo_audit = None;
        let mut crate_name = None;
        let mut source = None;
        let mut build = None;
        let mut r#type = None;
        let mut id = None;
        let mut workspace = None;
        while let Some(key) = map.next_key::<String>()? {
            match &*key {
                "package" => package = Some(map.next_value()?),
                "crate" => crate_name = Some(map.next_value()?),
                "source" => source = Some(map.next_value()?),
                "candid" => candid = Some(map.next_value()?),
                "build" => build = Some(map.next_value()?),
                "wasm" => wasm = Some(map.next_value()?),
                "type" => r#type = Some(map.next_value::<String>()?),
                "id" => id = Some(map.next_value()?),
                "workspace" => workspace = Some(map.next_value()?),
                "skip_cargo_audit" => skip_cargo_audit = Some(map.next_value()?),
                _ => continue,
            }
        }
        let props = match r#type.as_deref() {
            Some("motoko") | None => CanisterTypeProperties::Motoko,
            Some("rust") => CanisterTypeProperties::Rust {
                candid: PathBuf::from(candid.ok_or_else(|| missing_field("candid"))?),
                package: package.ok_or_else(|| missing_field("package"))?,
                skip_cargo_audit: skip_cargo_audit.unwrap_or(false),
                crate_name,
            },
            Some("assets") => CanisterTypeProperties::Assets {
                source: source.ok_or_else(|| missing_field("source"))?,
                build: build.unwrap_or_default(),
                workspace,
            },
            Some("custom") => CanisterTypeProperties::Custom {
                build: build.unwrap_or_default(),
                candid: candid.ok_or_else(|| missing_field("candid"))?,
                wasm: wasm.ok_or_else(|| missing_field("wasm"))?,
            },
            Some("pull") => CanisterTypeProperties::Pull {
                id: id.ok_or_else(|| missing_field("id"))?,
            },
            Some(x) => return Err(A::Error::unknown_variant(x, &BUILTIN_CANISTER_TYPES)),
        };
        Ok(props)
    }
}

#[derive(Clone)]
pub struct NetworksConfig {
    path: PathBuf,
    json: Value,
    // public interface to the networks config:
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
        let dir = get_user_dfx_config_dir().map_err(GetNetworkConfigPathFailed)?;

        let path = dir.join("networks.json");
        if path.exists() {
            NetworksConfig::from_file(&path).map_err(LoadNetworkConfigFromFileFailed)
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
        let content = crate::fs::read(path)?;

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

pub struct ToolConfig {
    path: PathBuf,
    json: Value,
    tool_config: ToolConfigInterface,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ToolConfigInterface {
    pub telemetry: TelemetryState,
}

impl ToolConfig {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn interface(&self) -> &ToolConfigInterface {
        &self.tool_config
    }

    pub fn interface_mut(&mut self) -> &mut ToolConfigInterface {
        &mut self.tool_config
    }

    pub fn new() -> Result<Self, ToolConfigError> {
        let dir = get_user_dfx_config_dir().map_err(GetToolConfigPathFailed)?;

        let path = dir.join("config.json");
        if path.exists() {
            Self::from_file(&path).map_err(LoadToolConfigFromFileFailed)
        } else {
            let default = Self {
                path,
                json: Default::default(),
                tool_config: ToolConfigInterface {
                    telemetry: TelemetryState::On,
                },
            };
            default.save()?;
            Ok(default)
        }
    }

    pub fn save(&self) -> Result<(), ToolConfigError> {
        self.save_to_file(&self.path)
            .map_err(SaveDefaultConfigFailed)
    }

    pub fn config_path(&self) -> &Path {
        &self.path
    }

    fn from_file(path: &Path) -> Result<Self, StructuredFileError> {
        let json: Value = load_json_file(path)?;
        let tool_config: ToolConfigInterface = serde_json::from_value(json.clone())
            .map_err(|e| DeserializeJsonFileFailed(Box::new(path.to_path_buf()), e))?;
        let path = PathBuf::from(path);
        Ok(Self {
            path,
            json,
            tool_config,
        })
    }

    fn save_to_file(&self, path: &Path) -> Result<(), StructuredFileError> {
        save_json_file(path, &self.tool_config)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, JsonSchema, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryState {
    On,
    Off,
    Local,
}

impl TelemetryState {
    pub fn should_collect(&self) -> bool {
        *self != TelemetryState::Off
    }
    pub fn should_publish(&self) -> bool {
        *self == TelemetryState::On
    }
}

impl Display for TelemetryState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(
            match self {
                Self::On => "on",
                Self::Off => "off",
                Self::Local => "local",
            },
            f,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_dfinity_config_current_path() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.keep().canonicalize().unwrap();
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
        let root_path = root_dir.keep().canonicalize().unwrap();
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
        let root_path = root_dir.keep().canonicalize().unwrap();
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
