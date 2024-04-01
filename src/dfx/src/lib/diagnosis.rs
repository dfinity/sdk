use crate::lib::error_code;
use anyhow::Error as AnyhowError;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::AgentError;
use ic_asset::error::{GatherAssetDescriptorsError, SyncError, UploadContentError};
use regex::Regex;
use std::path::Path;
use thiserror::Error as ThisError;

/// Contains two Option<Strings> that can be displayed to the user:
///   - Error explanation: Goes into a bit of detail on what the error is and/or where the user can find out more about it.
///   - Action suggestion: Tells the user how to move forward to resolve the error.
pub type Diagnosis = (Option<String>, Option<String>);
pub const NULL_DIAGNOSIS: Diagnosis = (None, None);

#[derive(ThisError, Debug)]
// This message will appear in the context trace of the stack. The diagnosis should not be displayed there yet.
#[error("Diagnosis was added here.")]
/// If you do not need the generic error diagnosis to run, you can add a DiagnosedError with .context(err: DiagnosedError).
/// In that case, no extra diagnosis is attempted and the last-added explanation and suggestion are printed out.
pub struct DiagnosedError {
    /// A user-friendly explanation of what went wrong.
    pub error_explanation: Option<String>,

    /// Suggestions for the user on how to move forward to recover from the error.
    pub action_suggestion: Option<String>,
}

impl DiagnosedError {
    pub fn new(error_explanation: String, action_suggestion: String) -> Self {
        Self {
            error_explanation: Some(error_explanation),
            action_suggestion: Some(action_suggestion),
        }
    }
}

/// Attempts to give helpful suggestions on how to resolve errors.
pub fn diagnose(err: &AnyhowError) -> Diagnosis {
    if let Some(diagnosed_error) = err.downcast_ref::<DiagnosedError>() {
        return (
            diagnosed_error.error_explanation.clone(),
            diagnosed_error.action_suggestion.clone(),
        );
    }

    if let Some(agent_err) = err.downcast_ref::<AgentError>() {
        if not_a_controller(agent_err) {
            return diagnose_http_403();
        }
    }

    if let Some(sync_error) = err.downcast_ref::<SyncError>() {
        if duplicate_asset_key_dist_and_src(sync_error) {
            return diagnose_duplicate_asset_key_dist_and_src();
        }
    }

    NULL_DIAGNOSIS
}

fn not_a_controller(err: &AgentError) -> bool {
    // Newer replicas include the error code in the reject response.
    if matches!(
        err,
        AgentError::UncertifiedReject(RejectResponse {
            reject_code: RejectCode::CanisterError,
            error_code: Some(error_code),
            ..
        }) if error_code == error_code::CANISTER_INVALID_CONTROLLER
    ) {
        return true;
    }

    // Older replicas do not include the error code in the reject response.
    // differing behavior between replica and ic-ref:
    // replica gives HTTP403, ic-ref gives HTTP400 with message "Wrong sender"
    matches!(err, AgentError::HttpError(payload) if payload.status == 403)
        || matches!(err, AgentError::HttpError(payload) if payload.status == 400 &&
            matches!(std::str::from_utf8(payload.content.as_slice()), Ok("Wrong sender")))
}

fn diagnose_http_403() -> Diagnosis {
    let error_explanation = "Each canister has a set of controllers. Only those controllers have access to the canister's management functions (like install_code or stop_canister).\n\
        The principal you are using to call a management function is not part of the controllers.";
    let action_suggestion = "To make the management function call succeed, you have to make sure the principal that calls the function is a controller.
To see the current controllers of a canister, use the 'dfx canister info (--network ic)' command.
To figure out which principal is calling the management function, look at the command you entered:
    If you used '--wallet <wallet id>', then the wallet's principal (the '<wallet id>') is calling the function.
    If you used '--no-wallet' or none of the flags, then your own principal is calling the function. You can see your own principal by running 'dfx identity get-principal'.
To add a principal to the list of controllers, one of the existing controllers has to add the new principal. The base command to do this is 'dfx canister update-settings --add-controller <controller principal to add> <canister id/name or --all> (--network ic)'.
If your wallet is a controller, but not your own principal, then you have to make your wallet perform the call by adding '--wallet <your wallet id>' to the command.

The most common way this error is solved is by running 'dfx canister update-settings --network ic --wallet \"$(dfx identity get-wallet)\" --all --add-controller \"$(dfx identity get-principal)\"'.";
    (
        Some(error_explanation.to_string()),
        Some(action_suggestion.to_string()),
    )
}

fn duplicate_asset_key_dist_and_src(sync_error: &SyncError) -> bool {
    fn is_src_to_dist(path0: &Path, path1: &Path) -> bool {
        // .../dist/<canister name>/... and .../src/<canister name>/assets/...
        let path0 = path0.to_string_lossy();
        let path1 = path1.to_string_lossy();
        let re = Regex::new(r"(?P<project_dir>.*)/dist/(?P<canister>[^/]*)/(?P<rest>.*)").unwrap();

        if let Some(caps) = re.captures(&path0) {
            let project_dir = caps["project_dir"].to_string();
            let canister = caps["canister"].to_string();
            let rest = caps["rest"].to_string();
            let transformed = format!("{}/src/{}/assets/{}", project_dir, canister, rest);
            return transformed == path1;
        }
        false
    }
    matches!(sync_error,
        SyncError::UploadContentFailed(
            UploadContentError::GatherAssetDescriptorsFailed(
                GatherAssetDescriptorsError::DuplicateAssetKey(_key, path0, path1)))
        if is_src_to_dist(path0, path1)
    )
}

fn diagnose_duplicate_asset_key_dist_and_src() -> Diagnosis {
    let explanation = "An asset key was found in both the dist and src directories.
One or both of the following are a likely explanation:
    - webpack.config.js is configured to copy assets from the src directory to the dist/ directory.
    - there are leftover files in the dist/ directory from a previous build.";
    let suggestion = r#"Perform the following steps:
    1. Remove the CopyPlugin step from webpack.config.js.  It looks like this:
        new CopyPlugin({
              patterns: [
                {
                  from: path.join(__dirname, "src", frontendDirectory, "assets"),
                  to: path.join(__dirname, "dist", frontendDirectory),
                },
              ],
            }),
    2. Delete all files from the dist/ directory."

See also release notes: https://forum.dfinity.org/t/dfx-0-11-0-is-promoted-with-breaking-changes/14327"#;

    (Some(explanation.to_string()), Some(suggestion.to_string()))
}
