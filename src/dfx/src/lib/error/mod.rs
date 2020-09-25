use ic_agent::AgentError;
use ic_types::principal::PrincipalError;

mod build;
mod cache;
mod config;
mod identity;

pub use build::BuildErrorKind;
pub use cache::CacheErrorKind;
pub use config::ConfigErrorKind;
pub use identity::IdentityErrorKind;
use serde::export::Formatter;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::PathBuf;

// TODO: refactor this enum into a *Kind enum and a struct DfxError.
#[derive(Debug)]
/// Provides dfx user facing errors.
pub enum DfxError {
    /// An error happened during build.
    BuildError(BuildErrorKind),

    /// An error happened while managing the cache.
    CacheError(CacheErrorKind),

    ConfigError(ConfigErrorKind),

    /// An error happened while managing identities.
    IdentityError(IdentityErrorKind),

    IdeError(String),

    Clap(clap::Error),
    Io(std::io::Error),
    IoWithPath(std::io::Error, std::path::PathBuf),
    Reqwest(reqwest::Error),

    CanisterNameMissing(),
    CannotFindCanisterName(String),
    CannotFindBuildOutputForCanister(String),

    /// There is already a dfx running in the background.
    DfxAlreadyRunningInBackground(),

    /// An unknown command was used. The argument is the command itself.
    UnknownCommand(String),

    /// Cannot create a new project because the directory already exists.
    ProjectExists,

    /// Not in a project.
    CommandMustBeRunInAProject,

    /// The agent returned an error (normally from the client).
    AgentError(AgentError),

    /// Could not generate a random principal for the purposes of
    /// building project in --check mode.
    CouldNotGenerateRandomPrincipal(PrincipalError),

    /// This option is used when the source/cause of the error is
    /// ambiguous. If the cause is known use or add a new option.
    Unknown(String),

    /// Configuration path does not exist in the config file.
    ConfigPathDoesNotExist(String),
    /// Argument provided is invalid.
    InvalidArgument(String),

    /// Configuration provided is invalid.
    InvalidConfiguration(String),
    /// Method called invalid.
    InvalidMethodCall(String),

    /// Data provided is invalid.
    InvalidData(String),
    RuntimeError(std::io::Error),

    /// Failed to clean up state.
    CleanState(std::io::Error, PathBuf),

    /// The ide server shouldn't be started from a terminal.
    LanguageServerFromATerminal,

    /// Configuration is invalid.
    CouldNotSerializeConfiguration(serde_json::error::Error),

    /// Generic IDL error.
    CouldNotSerializeIdlFile(candid::Error),

    /// An error during parsing of a version string.
    VersionCouldNotBeParsed(semver::SemVerError),

    /// A replica did not start successfully.
    ReplicaCouldNotBeStarted(),

    /// A canister in the dfx.json did not have a supported builder.
    CouldNotFindBuilderForCanister(String),

    /// Could not convert an OsString to a String
    CouldNotConvertOsString(OsString),

    /// A canister has an unsupported type.
    InvalidCanisterType(String),

    /// A canister name could not be found in the project.
    UnknownCanisterNamed(String),

    /// An error while traversing a directory tree
    CouldNotWalkDirectory(walkdir::Error),

    /// A directory lies outside the workspace root, and t
    DirectoryIsOutsideWorkspaceRoot(PathBuf),

    /// Could not parse an URL for some reason.
    InvalidUrl(String, url::ParseError),

    /// The value of the --network argument was not set.
    ComputeNetworkNotSet,

    /// The value of the --network argument was not found in dfx.json.
    ComputeNetworkNotFound(String),

    /// The network was found in dfx.json, but its "providers" array is empty.
    ComputeNetworkHasNoProviders(String),

    /// The "local" network provider with a bind address was not found.
    NoLocalNetworkProviderFound,

    /// The canister id was not found for the network
    CouldNotFindCanisterIdForNetwork(String, String),

    /// The canister name (when looked up by id) was not found for the network
    CouldNotFindCanisterNameForNetwork(String, String),

    /// Could not load the contents of the file
    CouldNotLoadCanisterIds(String, std::io::Error),

    /// Could not save the contents of the file
    CouldNotSaveCanisterIds(String, std::io::Error),

    HumanizeParseError(humanize_rs::ParseError),
}

/// The result of running a DFX command.
pub type DfxResult<T = ()> = Result<T, DfxError>;

impl Display for DfxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DfxError::BuildError(err) => {
                f.write_fmt(format_args!("Build failed. Reason:\n  {}", err))?;
            }
            DfxError::ConfigError(err) => {
                f.write_fmt(format_args!("Config error:\n  {}", err))?;
            }
            DfxError::IdentityError(err) => {
                f.write_fmt(format_args!("Identity error:\n  {}", err))?;
            }
            DfxError::IdeError(msg) => {
                f.write_fmt(format_args!(
                    "The Motoko Language Server returned an error:\n{}",
                    msg
                ))?;
            }
            DfxError::UnknownCommand(command) => {
                f.write_fmt(format_args!("Unknown command: {}", command))?;
            }
            DfxError::ProjectExists => {
                f.write_fmt(format_args!(
                    "Cannot create a new project because the directory already exists."
                ))?;
            }
            DfxError::CommandMustBeRunInAProject => {
                f.write_fmt(format_args!(
                    "Command must be run in a project directory (with a dfx.json file)."
                ))?;
            }
            DfxError::AgentError(AgentError::ReplicaError {
                reject_code,
                reject_message,
            }) => {
                f.write_fmt(format_args!(
                    "Replica error (code {}): {}",
                    reject_code, reject_message
                ))?;
            }
            DfxError::Unknown(err) => {
                f.write_fmt(format_args!("Unknown error: {}", err))?;
            }
            DfxError::ConfigPathDoesNotExist(config_path) => {
                f.write_fmt(format_args!("Config path does not exist: {}", config_path))?;
            }
            DfxError::InvalidArgument(e) => {
                f.write_fmt(format_args!("Invalid argument: {}", e))?;
            }
            DfxError::InvalidConfiguration(e) => {
                f.write_fmt(format_args!("Invalid configuration: {}", e))?;
            }
            DfxError::InvalidData(e) => {
                f.write_fmt(format_args!("Invalid data: {}", e))?;
            }
            DfxError::LanguageServerFromATerminal => {
                f.write_str(
                    "The `_language-service` command is meant to be run by editors to start a language service. You probably don't want to run it from a terminal.\nIf you _really_ want to, you can pass the --force-tty flag.",
                )?;
            }
            DfxError::ComputeNetworkNotSet => {
                f.write_str("Expected to find a network context, but found none")?;
            }
            DfxError::NoLocalNetworkProviderFound => {
                f.write_str("Expected there to be a local network with a bind address")?;
            }
            DfxError::CouldNotFindCanisterIdForNetwork(canister, network) => {
                let non_default_network = if network == "local" {
                    format!("")
                } else {
                    format!("--network {} ", network)
                };
                f.write_fmt(format_args!(
                    "Cannot find canister id.  Please issue 'dfx canister {}create {}'.",
                    non_default_network, canister
                ))?;
            }
            DfxError::CouldNotFindCanisterNameForNetwork(canister_id, network) => {
                f.write_fmt(format_args!(
                    "Cannot find canister id {} in network '{}'.",
                    canister_id, network
                ))?;
            }
            DfxError::CouldNotLoadCanisterIds(path, error) => {
                f.write_fmt(format_args!("Failed to load {} due to: {}", path, error))?;
            }
            DfxError::CouldNotSaveCanisterIds(path, error) => {
                f.write_fmt(format_args!("Failed to save {} due to: {}", path, error))?;
            }
            err => {
                f.write_fmt(format_args!("An error occured:\n{:#?}", err))?;
            }
        }
        Ok(())
    }
}

impl From<clap::Error> for DfxError {
    fn from(err: clap::Error) -> DfxError {
        DfxError::Clap(err)
    }
}

impl From<AgentError> for DfxError {
    fn from(err: AgentError) -> DfxError {
        DfxError::AgentError(err)
    }
}

impl From<PrincipalError> for DfxError {
    fn from(err: PrincipalError) -> DfxError {
        DfxError::CouldNotGenerateRandomPrincipal(err)
    }
}

impl From<reqwest::Error> for DfxError {
    fn from(err: reqwest::Error) -> DfxError {
        DfxError::Reqwest(err)
    }
}

impl From<std::io::Error> for DfxError {
    fn from(err: std::io::Error) -> DfxError {
        DfxError::Io(err)
    }
}

impl From<serde_json::error::Error> for DfxError {
    fn from(err: serde_json::error::Error) -> DfxError {
        DfxError::CouldNotSerializeConfiguration(err)
    }
}

impl From<candid::error::Error> for DfxError {
    fn from(err: candid::error::Error) -> Self {
        DfxError::CouldNotSerializeIdlFile(err)
    }
}

impl From<semver::SemVerError> for DfxError {
    fn from(err: semver::SemVerError) -> DfxError {
        DfxError::VersionCouldNotBeParsed(err)
    }
}

impl From<walkdir::Error> for DfxError {
    fn from(err: walkdir::Error) -> DfxError {
        DfxError::CouldNotWalkDirectory(err)
    }
}

impl actix_web::error::ResponseError for DfxError {}

impl From<std::string::String> for DfxError {
    fn from(err: std::string::String) -> DfxError {
        DfxError::Unknown(err)
    }
}

impl From<humanize_rs::ParseError> for DfxError {
    fn from(err: humanize_rs::ParseError) -> DfxError {
        DfxError::HumanizeParseError(err)
    }
}
