use crate::lib::error::DfxError;
use ic_agent::CanisterId;
use std::fmt;
use std::io::Error;
use std::process::ExitStatus;

/// An error happened during build.
#[derive(Debug)]
pub enum BuildErrorKind {
    /// The prebuild all step failed with the embedded error.
    PrebuildAllStepFailed(Box<DfxError>),

    /// The prebuild all step failed with the embedded error.
    PostbuildAllStepFailed(Box<DfxError>),

    /// The prebuild step failed with the embedded error.
    PrebuildStepFailed(CanisterId, Box<DfxError>),

    /// The prebuild all step failed with the embedded error.
    BuildStepFailed(CanisterId, Box<DfxError>),

    /// The prebuild step failed with the embedded error.
    PostbuildStepFailed(CanisterId, Box<DfxError>),

    /// A compiler error happened.
    CompilerError(String, String, String),

    /// An error happened while dependency analysis.
    DependencyError(String),

    /// An error happened while creating the JS canister bindings.
    CanisterJsGenerationError(String),

    // A cycle was detected in the dependency between canisters. For now we don't have
    // a list of dependencies creating the cycle.
    CircularDependency(String),

    /// An error happened while trying to invoke the package tool.
    FailedToInvokePackageTool(String, Error),

    /// Ran the package tool, but it reported an error
    PackageToolReportedError(String, ExitStatus, String, String),

    /// An custom tool failed. See description above for why.
    CustomToolError(Option<i32>),

    /// A command line string was invalid.
    InvalidBuildCommand(String),
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
            CircularDependency(name) => f.write_fmt(format_args!(
                "There is a dependency cycle between canisters found at canister {}.",
                name,
            )),
            FailedToInvokePackageTool(cmd, error) => f.write_fmt(format_args!(
                "Failed to invoke the package tool {}\n the error was: {}",
                cmd, error
            )),
            PackageToolReportedError(cmd, exit_status, stdout, stderr) => {
                f.write_fmt(format_args!(
                    "Package tool {}\n reported an error: {}\n{}{}",
                    cmd, exit_status, stdout, stderr
                ))
            }
            InvalidBuildCommand(_) => {
                f.write_fmt(format_args!("Build command could not be parsed."))
            }
            CustomToolError(status) => match status {
                None => f.write_str("Custom tool interrupted by signal."),
                Some(code) => f.write_fmt(format_args!(
                    "A custom tool failed with status {}. See above for more information.",
                    code
                )),
            },
            PrebuildAllStepFailed(e) => {
                f.write_fmt(format_args!("Prebuild ALL step failed with error: {}", e))
            }

            PostbuildAllStepFailed(e) => {
                f.write_fmt(format_args!("Postbuild ALL step failed with error: {}", e))
            }

            PrebuildStepFailed(c, e) => f.write_fmt(format_args!(
                "Prebuild step failed for canister {} with error: {}",
                c, e
            )),

            BuildStepFailed(c, e) => f.write_fmt(format_args!(
                "Build step failed for canister {} with error: {}",
                c, e
            )),

            PostbuildStepFailed(c, e) => f.write_fmt(format_args!(
                "Postbuild step failed for canister {} with error: {}",
                c, e
            )),
        }
    }
}
