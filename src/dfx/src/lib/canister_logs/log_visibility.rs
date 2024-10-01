use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::clap::parsers::{log_visibility_parser, principal_parser};
use anyhow::anyhow;
use candid::Principal;
use clap::{ArgAction, Args};
use dfx_core::cli::ask_for_consent;
use ic_utils::interfaces::management_canister::{LogVisibility, StatusCallResult};

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
    #[arg(long, action = ArgAction::Append, value_parser = principal_parser, conflicts_with("set_log_viewer"))]
    add_log_viewer: Option<Vec<Principal>>,

    /// Removes a principal from the list of log viewers of the canister.
    #[arg(long, action = ArgAction::Append, value_parser = principal_parser, conflicts_with("set_log_viewer"))]
    remove_log_viewer: Option<Vec<Principal>>,

    /// Specifies the the principal of the log viewer of the canister.
    /// Can be specified more than once.
    #[arg(
        long,
        action = ArgAction::Append,
        value_parser = principal_parser,
        conflicts_with("add_log_viewer"),
        conflicts_with("remove_log_viewer"),
    )]
    set_log_viewer: Option<Vec<Principal>>,
}

impl LogVisibilityOpt {
    pub fn require_current_settings(&self) -> bool {
        self.add_log_viewer.is_some() || self.remove_log_viewer.is_some()
    }

    pub fn from(
        log_visibility: &Option<LogVisibility>,
        log_viewer: &Option<Vec<Principal>>,
    ) -> Option<LogVisibilityOpt> {
        if let Some(log_visibility) = log_visibility {
            return Some(LogVisibilityOpt {
                log_visibility: Some(log_visibility.clone()),
                add_log_viewer: None,
                remove_log_viewer: None,
                set_log_viewer: None,
            });
        }

        if let Some(log_viewer) = log_viewer {
            return Some(LogVisibilityOpt {
                log_visibility: None,
                add_log_viewer: None,
                remove_log_viewer: None,
                set_log_viewer: Some(log_viewer.clone()),
            });
        }

        None
    }

    pub fn to_log_visibility(
        &self,
        env: &dyn Environment,
        current_status: Option<&StatusCallResult>,
    ) -> DfxResult<LogVisibility> {
        let logger = env.get_logger();

        // For public and controllers.
        if let Some(log_visibility) = self.log_visibility.as_ref() {
            return Ok(log_visibility.clone());
        }

        // For setting viewers.
        if let Some(principals) = self.set_log_viewer.as_ref() {
            return Ok(LogVisibility::AllowedViewers(principals.clone()));
        }

        // Get the current viewer list for adding and removing, only for update-settings.
        let mut current_visibility: Option<LogVisibility> = None;
        let mut viewers = match current_status {
            Some(status) => {
                current_visibility = Some(status.settings.log_visibility.clone());
                match &status.settings.log_visibility {
                    LogVisibility::AllowedViewers(viewers) => viewers.clone(),
                    _ => vec![],
                }
            }
            None => vec![],
        };

        // Adding.
        if let Some(added) = self.add_log_viewer.as_ref() {
            if let Some(LogVisibility::Public) = current_visibility {
                let msg = "Current log is public to everyone. Adding log reviewers will make the log only visible to the reviewers.";
                ask_for_consent(msg)?;
            }

            for principal in added {
                if !viewers.iter().any(|x| x == principal) {
                    viewers.push(*principal);
                }
            }
        }

        // Removing.
        if let Some(removed) = self.remove_log_viewer.as_ref() {
            if let Some(visibility) = &current_visibility {
                match visibility {
                    LogVisibility::Public | LogVisibility::Controllers => {
                        return Err(anyhow!("Removing reviewers is not allowed with 'public' or 'controllers' log visibility."));
                    }
                    _ => (),
                }
            }
            for principal in removed {
                if let Some(idx) = viewers.iter().position(|x| x == principal) {
                    viewers.swap_remove(idx);
                } else {
                    slog::warn!(
                        logger,
                        "Principal '{}' is not in the allowed list.",
                        principal.to_text()
                    );
                }
            }
        }

        Ok(LogVisibility::AllowedViewers(viewers))
    }
}
