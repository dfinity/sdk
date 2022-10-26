use anyhow::Error as AnyhowError;
use ic_agent::AgentError;
use thiserror::Error as ThisError;

use super::environment::Environment;

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
pub fn diagnose(_env: &dyn Environment, err: &AnyhowError) -> Diagnosis {
    if let Some(diagnosed_error) = err.downcast_ref::<DiagnosedError>() {
        (
            diagnosed_error.error_explanation.clone(),
            diagnosed_error.action_suggestion.clone(),
        )
    } else if let Some(agent_err) = err.downcast_ref::<AgentError>() {
        match agent_err {
            AgentError::HttpError(payload) => match payload.status {
                403 => diagnose_http_403(),
                400 => match std::str::from_utf8(payload.content.as_slice()) {
                    Ok("Wrong sender") => {
                        // differing behavior between replica and ic-ref:
                        // replica gives HTTP403, ic-ref gives HTTP400 with message "Wrong sender"
                        diagnose_http_403()
                    }
                    _ => NULL_DIAGNOSIS,
                },
                _ => NULL_DIAGNOSIS,
            },
            _ => NULL_DIAGNOSIS,
        }
    } else {
        NULL_DIAGNOSIS
    }
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
