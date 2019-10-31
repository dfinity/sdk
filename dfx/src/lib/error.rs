use std::fmt;

#[derive(Debug)]
/// An error happened during build.
pub enum BuildErrorKind {
    /// Invalid extension.
    InvalidExtension(String),

    /// A compiler error happened.
    MotokoCompilerError(String),

    /// An error happened during the generation of the Idl.
    IdlGenerationError(String),

    /// An error happened while generating the user library.
    UserLibGenerationError(String),

    /// An error happened while compiling WAT to WASM.
    WatCompileError(wabt::Error),
}

impl fmt::Display for BuildErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BuildErrorKind::*;

        match self {
            InvalidExtension(ext) => {
                f.write_fmt(format_args!("Invalid extension: {}", ext))?;
            }
            MotokoCompilerError(stdout) => {
                f.write_fmt(format_args!("Motoko returned an error:\n{}", stdout))?;
            }
            IdlGenerationError(stdout) => {
                f.write_fmt(format_args!(
                    "IDL generation returned an error:\n{}",
                    stdout
                ))?;
            }
            UserLibGenerationError(stdout) => {
                f.write_fmt(format_args!(
                    "UserLib generation returned an error:\n{}",
                    stdout
                ))?;
            }
            WatCompileError(e) => {
                f.write_fmt(format_args!("Error while compiling WAT to WASM: {}", e))?;
            }
        };

        Ok(())
    }
}

// TODO: refactor this enum into a *Kind enum and a struct DfxError.
#[derive(Debug)]
pub enum DfxError {
    /// An error happened during build.
    BuildError(BuildErrorKind),
    Clap(clap::Error),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Url(reqwest::UrlError),

    /// An unknown command was used. The argument is the command itself.
    UnknownCommand(String),

    // Cannot create a new project because the directory already exists.
    ProjectExists,

    // Not in a project.
    CommandMustBeRunInAProject,

    // The client returned an error. It normally specifies the error as an
    // HTTP status (so 400-599), and has a string as the error message.
    // Once the client support errors from the public spec or as an enum,
    // we should update this type.
    // We don't use StatusCode here because the client might return some other
    // number if they support public spec's errors (< 100).
    ClientError(u16, String),
    Unknown(String),

    // Configuration path does not exist in the config file.
    ConfigPathDoesNotExist(String),
    InvalidArgument(String),
    InvalidData(String),
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
