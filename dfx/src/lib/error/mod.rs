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
    Url(reqwest::UrlError),

    CanisterNameMissing(),
    CannotFindCanisterName(String),
    CannotFindBuildOutputForCanister(String),

    /// There is already a dfx running in the background.
    DfxAlreadyRunningInBackground(),

    /// An unknown command was used. The argument is the command itself.
    UnknownCommand(String),

    /// Cannot create a new project because the directory already exists.
    ProjectExists,

    /// An error originating from the IC. The enclosed type should be
    /// a descriptive error.
    // TODO(eftychis): Consider to how to better represent this without a massive change.
    ICContainerError(String),

    /// Not in a project.
    CommandMustBeRunInAProject,

    /// The client returned an error. It normally specifies the error as an
    /// HTTP status (so 400-599), and has a string as the error message.
    /// Once the client support errors from the public spec or as an enum,
    /// we should update this type.
    /// We don't use StatusCode here because the client might return some other
    /// number if they support public spec's errors (< 100).
    ClientError(u16, String),

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

    /// The ide server shouldn't be started from a terminal.
    LanguageServerFromATerminal,
}

/// The result of running a DFX command.
pub type DfxResult<T = ()> = Result<T, DfxError>;

impl From<clap::Error> for DfxError {
    fn from(err: clap::Error) -> DfxError {
        DfxError::Clap(err)
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
