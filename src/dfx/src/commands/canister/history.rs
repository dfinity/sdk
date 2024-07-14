use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use dfx_core::identity::CallSender;

/// Prints the url of a canister.
#[derive(Parser)]
pub struct CanisterHistoryOpts {
    /// Specifies the name or id of the canister.
    canister: String,

    num_requested_changes: Option<u64>,
}

pub async fn exec(_env: &dyn Environment, 
    _opts: CanisterHistoryOpts,
    _call_sender: &CallSender,
) -> DfxResult {
    println!("Is called");
    Ok(())
}
