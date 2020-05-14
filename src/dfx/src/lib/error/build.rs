use std::fmt;

/// An error happened during build.
#[derive(Debug)]
pub enum BuildErrorKind {
    /// A compiler error happened.
    CompilerError(String, String, String),

    /// An error happened while dependency analysis.
    DependencyError(String),

    /// An error happened while creating the JS canister bindings.
    CanisterJsGenerationError(String),

    // Cannot find or read the canister ID.
    CouldNotReadCanisterId(),

    // A cycle was detected in the dependency between canisters. For now we don't have
    // a list of dependencies creating the cycle.
    CircularDependency(String),
}

impl fmt::Display for BuildErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BuildErrorKind::*;

        match self {
            CompilerError(cmd, stdout, stderr) => f.write_fmt(format_args!(
                "Command {}\n returned an error:\n{}{}",
                cmd, stdout, stderr
            )),
            DependencyError(msg) => f.write_fmt(format_args!(
                "Error while performing dependency analysis: {}",
                msg
            )),
            CanisterJsGenerationError(stdout) => f.write_fmt(format_args!(
                "Creating canister JS bindings returned an error:\n{}",
                stdout
            )),
            CouldNotReadCanisterId() => f.write_str("The canister ID could not be found."),
            CircularDependency(name) => f.write_fmt(format_args!(
                "There is a dependency cycle between canisters found at canister {}.",
                name,
            )),
        }
    }
}
