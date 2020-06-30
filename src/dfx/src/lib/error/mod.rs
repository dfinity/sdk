use ic_agent::AgentError;

mod build;
mod cache;

pub use build::BuildErrorKind;
pub use cache::CacheErrorKind;
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

    IdeError(String),

    Clap(clap::Error),
    Io(std::io::Error),
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

    /// This option is used when the source/cause of the error is
    /// ambiguous. If the cause is known use or add a new option.
    Unknown(String),

    /// Configuration path does not exist in the config file.
    ConfigPathDoesNotExist(String),
    /// Argument provided is invalid.
    InvalidArgument(String),

    #[allow(dead_code)]
    /// Configuration provided is invalid.
    InvalidConfiguration(String),
    /// Method called invalid.
    InvalidMethodCall(String),

    /// Data provided is invalid.
    InvalidData(String),
    RuntimeError(std::io::Error),

    /// Failed to clean up state.
    CleanState(std::io::Error),

    /// The ide server shouldn't be started from a terminal.
    LanguageServerFromATerminal,

    /// Configuration is invalid.
    CouldNotSerializeConfiguration(serde_json::error::Error),

    /// Generic IDL error.
    CouldNotSerializeIdlFile(candid::Error),

    /// An error during parsing of a version string.
    VersionCouldNotBeParsed(semver::SemVerError),

    /// String provided is not a port
    CouldNotParsePort(std::num::ParseIntError),

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

    /// The value of the --network argument was not found in dfx.json.
    ComputeNetworkNotFound(String),

    /// The network was found in dfx.json, but its "providers" array is empty.
    ComputeNetworkHasNoProviders(String),
}

/// The result of running a DFX command.
pub type DfxResult<T = ()> = Result<T, DfxError>;

impl Display for DfxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DfxError::BuildError(err) => {
                f.write_fmt(format_args!("Build failed. Reason:\n  {}", err))?;
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
            DfxError::InvalidData(e) => {
                f.write_fmt(format_args!("Invalid data: {}", e))?;
            }
            DfxError::LanguageServerFromATerminal => {
                f.write_str(
                    "The `_language-service` command is meant to be run by editors to start a language service. You probably don't want to run it from a terminal.\nIf you _really_ want to, you can pass the --force-tty flag.",
                )?;
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
