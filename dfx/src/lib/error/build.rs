use std::fmt;

/// An error happened during build.
#[derive(Debug)]
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
            InvalidExtension(ext) => f.write_fmt(format_args!("Invalid extension: {}", ext)),
            MotokoCompilerError(stdout) => {
                f.write_fmt(format_args!("Motoko returned an error:\n{}", stdout))
            }
            IdlGenerationError(stdout) => f.write_fmt(format_args!(
                "IDL generation returned an error:\n{}",
                stdout
            )),
            UserLibGenerationError(stdout) => f.write_fmt(format_args!(
                "UserLib generation returned an error:\n{}",
                stdout
            )),
            WatCompileError(e) => {
                f.write_fmt(format_args!("Error while compiling WAT to WASM: {}", e))
            }
        }
    }
}
