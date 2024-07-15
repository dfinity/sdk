use crate::commands::canister::call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::clap::argument_from_cli::ArgumentFromCliPositionalOpt;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;

/// Prints the history of a canister.
#[derive(Parser)]
pub struct CanisterHistoryOpts {
    /// Specifies the name or id of the canister.
    canister: String,

    /// Specifies the number of recent changes of the canister.
    /// The maximum 20 will be used if not specified.
    num_requested_changes: Option<u64>,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterHistoryOpts,
    call_sender: &CallSender,
) -> DfxResult {
    // Get the canister id.
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id = Principal::from_text(opts.canister.as_str())
        .or_else(|_| canister_id_store.get(opts.canister.as_str()))?;

    // Composite the argument for the proxy canister call.
    let argument = format!(
        "(record {{canister_id=principal \"{}\"; num_requested_changes=opt {}}})",
        canister_id,
        opts.num_requested_changes.unwrap_or(20)
    );

    let call_opts = call::CanisterCallOpts {
        canister_name: String::from("pxmfj-jaaaa-aaaan-qmmbq-cai"), // The proxy canister.
        method_name: String::from("canister_history"),              // The proxy method.
        argument_from_cli: ArgumentFromCliPositionalOpt {
            argument: Some(argument),
            r#type: None,
            argument_file: None,
        },
        r#async: false,
        query: false,
        update: false,
        random: None,
        output: None,
        with_cycles: None,
        candid: None,
        always_assist: false,
    };

    return call::exec(env, call_opts, call_sender).await;
}
