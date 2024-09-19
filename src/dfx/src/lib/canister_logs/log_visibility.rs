use crate::lib::error::DfxResult;
use crate::util::clap::parsers::log_visibility_parser;
use candid::Principal;
use clap::{ArgAction, Args};
use ic_utils::interfaces::management_canister::LogVisibility;

#[derive(Args, Clone, Debug, Default)]
pub struct LogVisibilityOpt {
    /// Specifies who is allowed to read the canister's logs.
    /// Can be either "controllers" or "public".
    #[arg(
        long,
        value_parser = log_visibility_parser,
        conflicts_with("add_log_viewer"),
        conflicts_with("remove_log_viewer"),
        conflicts_with("set_log_viewer"),
    )]
    log_visibility: Option<LogVisibility>,

    /// Add a principal to the list of log viewers of the canister.
    #[arg(long, action = ArgAction::Append, conflicts_with("set_log_viewer"))]
    add_log_viewer: Option<Vec<String>>,

    /// Remove a principal from the list of log viewers of the canister.
    #[arg(long, action = ArgAction::Append, conflicts_with("set_log_viewer"))]
    remove_log_viewer: Option<Vec<String>>,

    /// Specifies the the principal of the log viewers of the canister.
    /// Can be specified more than once.
    #[arg(
        long,
        action = ArgAction::Append,
        conflicts_with("add_log_viewer"),
        conflicts_with("remove_log_viewer"),
    )]
    set_log_viewer: Option<Vec<String>>,
}

impl LogVisibilityOpt {
    pub fn to_log_visibility(&self) -> Result<LogVisibility, String> {
        if let Some(log_visibility) = self.log_visibility.as_ref() {
            return Ok(log_visibility.clone());
        }

        // TODO: Get the current viewer list.
        // The below is just a POC, next will handle removed/set.

        if let Some(added) = self.add_log_viewer.as_ref() {
            let principals: DfxResult<Vec<_>> = added
                .iter()
                .map(|viewer| Ok(Principal::from_text(viewer).unwrap()))
                .collect();

            let ids = principals.unwrap();
            return Ok(LogVisibility::AllowedViewers(ids));
        }

        Err("Faile to convert to LogVisibility.".to_string())
    }
}
