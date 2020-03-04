use ic_http_agent::AgentError;

mod build;
mod cache;

pub use build::BuildErrorKind;
pub use cache::CacheErrorKind;

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

    /// Client TOML Serialization error.
    CouldNotSerializeClientConfiguration(toml::ser::Error),

    /// An error during parsing of a version string.
    VersionCouldNotBeParsed(semver::SemVerError),

    /// String provided is not a port
    CouldNotParsePort(std::num::ParseIntError),
}

/// The result of running a DFX command.
pub type DfxResult<T = ()> = Result<T, DfxError>;

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

impl From<semver::SemVerError> for DfxError {
    fn from(err: semver::SemVerError) -> DfxError {
        DfxError::VersionCouldNotBeParsed(err)
    }
}
