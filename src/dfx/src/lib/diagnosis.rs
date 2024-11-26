use crate::lib::cycles_ledger_types::create_canister::CreateCanisterError;
use crate::lib::error_code;
use anyhow::Error as AnyhowError;
use dfx_core::error::root_key::FetchRootKeyError;
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
        if wallet_method_not_found(agent_err) {
            return diagnose_bad_wallet();
        }
        if not_a_controller(agent_err) {
            return diagnose_http_403();
        } else if *agent_err == AgentError::CertificateNotAuthorized() {
            return subnet_not_authorized();
        }
        if cycles_ledger_not_found(err) {
            return diagnose_cycles_ledger_not_found();
        }
        if ledger_not_found(err) {
            return diagnose_ledger_not_found();
        }
    }

    if local_replica_not_running(err) {
        return diagnose_local_replica_not_running();
    }

    if let Some(sync_error) = err.downcast_ref::<SyncError>() {
        if duplicate_asset_key_dist_and_src(sync_error) {
            return diagnose_duplicate_asset_key_dist_and_src();
        }
    }

    if let Some(_create_canister_err) = err.downcast_ref::<CreateCanisterError>() {
        if insufficient_cycles(_create_canister_err) {
            return diagnose_insufficient_cycles();
        }
    }

    NULL_DIAGNOSIS
}

fn local_replica_not_running(err: &AnyhowError) -> bool {
    let maybe_agent_error = {
        if let Some(FetchRootKeyError::AgentError(agent_error)) =
            err.downcast_ref::<FetchRootKeyError>()
        {
            Some(agent_error)
        } else {
            err.downcast_ref::<AgentError>()
        }
    };
    if let Some(AgentError::TransportError(transport_error)) = maybe_agent_error {
        transport_error.is_connect()
            && transport_error
                .url()
                .and_then(|url| url.host())
                .map(|host| match host {
                    url::Host::Domain(domain) => domain == "localhost",
                    url::Host::Ipv4(ipv4_addr) => ipv4_addr.is_loopback(),
                    url::Host::Ipv6(ipv6_addr) => ipv6_addr.is_loopback(),
                })
                .unwrap_or(false)
    } else {
        false
    }
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

fn wallet_method_not_found(err: &AgentError) -> bool {
    match err {
        AgentError::CertifiedReject(RejectResponse {
            reject_code: RejectCode::CanisterError,
            reject_message,
            ..
        }) if reject_message.contains("Canister has no update method 'wallet_") => true,
        AgentError::UncertifiedReject(RejectResponse {
            reject_code: RejectCode::CanisterError,
            reject_message,
            ..
        }) if reject_message.contains("Canister has no query method 'wallet_") => true,
        _ => false,
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

fn diagnose_local_replica_not_running() -> Diagnosis {
    let error_explanation =
        "You are trying to connect to the local replica but dfx cannot connect to it.";
    let action_suggestion =
        "Target a different network or run 'dfx start' to start the local replica.";
    (
        Some(error_explanation.to_string()),
        Some(action_suggestion.to_string()),
    )
}

fn subnet_not_authorized() -> Diagnosis {
    let action_suggestion = "If you are connecting to a node directly instead of a boundary node, try using --provisional-create-canister-effective-canister-id with a canister id in the subnet's canister range. First non-root subnet: 5v3p4-iyaaa-aaaaa-qaaaa-cai, second non-root subnet: jrlun-jiaaa-aaaab-aaaaa-cai";
    (None, Some(action_suggestion.to_string()))
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

fn diagnose_bad_wallet() -> Diagnosis {
    let explanation = "\
A wallet has been previously configured (e.g. via `dfx identity set-wallet`).
However, it did not contain a function that dfx was looking for.
This may be because:
    - a wallet was correctly installed, but is outdated
    - `dfx identity set-wallet` was used on a non-wallet canister";
    let suggestion = "\
If you have had the wallet for a while, then you may need to update it with
`dfx wallet upgrade`. The release notes indicate when there is a new wallet.
If you recently ran `dfx identity set-wallet`, and the canister may have been
wrong, you can set a new wallet with
`dfx identity set-wallet <PRINCIPAL> --identity <IDENTITY>`.
If you're using a local replica and configuring a wallet was a mistake, you can
recreate the replica with `dfx stop && dfx start --clean` to start over.";
    (Some(explanation.to_string()), Some(suggestion.to_string()))
}

fn cycles_ledger_not_found(err: &AnyhowError) -> bool {
    err.to_string()
        .contains("Canister um5iw-rqaaa-aaaaq-qaaba-cai not found")
}

fn diagnose_cycles_ledger_not_found() -> Diagnosis {
    let explanation =
        "Cycles ledger with canister ID 'um5iw-rqaaa-aaaaq-qaaba-cai' is not installed.";
    let suggestion =
        "Run the command with '--ic' flag if you want to manage the cycles on the mainnet.";

    (Some(explanation.to_string()), Some(suggestion.to_string()))
}

fn ledger_not_found(err: &AnyhowError) -> bool {
    err.to_string()
        .contains("Canister ryjl3-tyaaa-aaaaa-aaaba-cai not found")
}

fn diagnose_ledger_not_found() -> Diagnosis {
    let explanation = "ICP Ledger with canister ID 'ryjl3-tyaaa-aaaaa-aaaba-cai' is not installed.";
    let suggestion =
        "Run the command with '--ic' flag if you want to manage the ICP on the mainnet.";

    (Some(explanation.to_string()), Some(suggestion.to_string()))
}

fn insufficient_cycles(err: &CreateCanisterError) -> bool {
    match err {
        CreateCanisterError::InsufficientFunds { balance: _ } => true,
        _ => false,
    }
}

fn diagnose_insufficient_cycles() -> Diagnosis {
    let explanation = "Insufficient cycles balance to create the canister.";
    let suggestion = "Please top up your cycles balance by converting ICP to cycles like below:
'dfx cycles convert --amount=0.123 --ic'.";
    (Some(explanation.to_string()), Some(suggestion.to_string()))
}
