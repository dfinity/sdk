use crate::lib::error::DfxError;

use candid::Principal;
use std::process::ExitStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("The pre-build all step failed with an embedded error: {0:#}")]
    PreBuildAllStepFailed(Box<DfxError>),

    #[error("The post-build all step failed with an embedded error: {0:#}")]
    PostBuildAllStepFailed(Box<DfxError>),

    #[error("The pre-build step failed for canister '{0}' ({1}) with an embedded error: {2:#}")]
    PreBuildStepFailed(Principal, String, Box<DfxError>),

    #[error("The build step failed for canister '{0}' ({1}) with an embedded error: {2:#}")]
    BuildStepFailed(Principal, String, Box<DfxError>),

    #[error("The post-build step failed for canister '{0}' ({1}) with an embedded error: {2:#}")]
    PostBuildStepFailed(Principal, String, Box<DfxError>),

    #[error("The command '{0}' failed with exit status '{1}'.\nStdout:\n{2}\nStderr:\n{3}")]
    CommandError(String, ExitStatus, String, String),

    #[error("The dependency analyzer failed: {0}")]
    DependencyError(String),

    #[error("The JavaScript bindings generator failed: {0}")]
    JsBindGenError(String),

    #[error("The custom tool failed.")]
    CustomToolError(Option<i32>),
}
