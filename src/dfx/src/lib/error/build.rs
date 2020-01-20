use std::fmt;

/// An error happened during build.
#[derive(Debug)]
pub enum BuildErrorKind {
    /// Invalid extension.
    InvalidExtension(String),

    /// A compiler error happened.
    CompilerError(String, String, String),

    /// An error happened while creating the JS canister bindings.
    CanisterJsGenerationError(String),

    /// An error happened while compiling WAT to WASM.
    WatCompileError(wabt::Error),

    /// Could not find the canister to build in the config.
    CanisterNameIsNotInConfigError(String),

    // The frontend failed.
    FrontendBuildError(),

    // Cannot find or read the canister ID.
    CouldNotReadCanisterId(),
}

impl fmt::Display for BuildErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BuildErrorKind::*;

        match self {
            InvalidExtension(ext) => f.write_fmt(format_args!("Invalid extension: {}", ext)),
            CompilerError(cmd, stdout, stderr) => f.write_fmt(format_args!(
                "Command {}\n returned an error:\n{}\n{}",
                cmd, stdout, stderr
            )),
            CanisterJsGenerationError(stdout) => f.write_fmt(format_args!(
                "Creating canister JS bindings returned an error:\n{}",
                stdout
            )),
            WatCompileError(e) => {
                f.write_fmt(format_args!("Error while compiling WAT to WASM: {}", e))
            }
            CanisterNameIsNotInConfigError(name) => f.write_fmt(format_args!(
                r#"Could not find the canister named "{}" in the dfx.json configuration."#,
                name,
            )),
            FrontendBuildError() => f.write_str("Frontend build stage failed."),
            CouldNotReadCanisterId() => f.write_str("The canister ID could not be found."),
        }
    }
}
