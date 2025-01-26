use crate::lib::error::DfxError;
use candid::Principal;
use std::process::ExitStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("The pre-build all step failed")]
    PreBuildAllStepFailed(#[source] Box<DfxError>),

    // #[error("The post-build all step failed")]
    // PostBuildAllStepFailed(#[source] Box<DfxError>),
    #[error("The pre-build step failed for canister '{0}' ({1})")]
    PreBuildStepFailed(Principal, String, #[source] Box<DfxError>),

    #[error("The build step failed for canister '{0}' ({1})")]
    BuildStepFailed(Principal, String, #[source] Box<DfxError>),

    #[error("The post-build step failed for canister '{0}' ({1})")]
    PostBuildStepFailed(Principal, String, #[source] Box<DfxError>),

    #[error("The command '{0}' failed with exit status '{1}'.\nStdout:\n{2}\nStderr:\n{3}")]
    CommandError(String, ExitStatus, String, String),

    #[error("The dependency analyzer failed: {0}")]
    DependencyError(String),

    #[error("The JavaScript bindings generator failed: {0}")]
    JsBindGenError(String),

    #[error("The custom tool failed.")]
    CustomToolError(Option<i32>),
}
