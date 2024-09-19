use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::get_canister_status;
use crate::util::clap::parsers::log_visibility_parser;
use candid::Principal;
use clap::{ArgAction, Args};
use dfx_core::identity::CallSender;
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
    pub async fn to_log_visibility(
        &self,
        env: &dyn Environment,
        canister_id: Option<Principal>,
        call_sender: &CallSender,
    ) -> Result<LogVisibility, String> {
        // For public and controllers.
        if let Some(log_visibility) = self.log_visibility.as_ref() {
            return Ok(log_visibility.clone());
        }

        // For setting viewers.
        if let Some(viewers) = self.set_log_viewer.as_ref() {
            let principals: DfxResult<Vec<_>> = viewers
                .iter()
                .map(|viewer| Ok(Principal::from_text(viewer).unwrap()))
                .collect();

            return Ok(LogVisibility::AllowedViewers(principals.unwrap()));
        }

        // Get the current viewer list for adding and removing, only for update-settings.
        let mut current_visibility: Option<LogVisibility> = None;
        let mut viewers = match canister_id {
            Some(id) => {
                let status = get_canister_status(env, id, call_sender).await.unwrap();
                current_visibility = Some(status.settings.log_visibility.clone());
                match status.settings.log_visibility {
                    LogVisibility::AllowedViewers(viewers) => viewers,
                    _ => vec![],
                }
            }
            None => vec![],
        };

        // Adding.
        if let Some(added) = self.add_log_viewer.as_ref() {
            if let Some(visibility) = &current_visibility {
                match visibility {
                    LogVisibility::Public => {
                        // TODO:
                        // Warning for taking away view rights for everyone.
                    }
                    _ => (),
                }
            }
            for viewer in added {
                let principal = Principal::from_text(viewer).unwrap();
                if let Some(_) = viewers.iter().position(|x| *x == principal) {
                    continue;
                }
                viewers.push(principal);
            }
        }

        // Removing.
        if let Some(removed) = self.remove_log_viewer.as_ref() {
            if let Some(visibility) = &current_visibility {
                match visibility {
                    LogVisibility::Public | LogVisibility::Controllers => {
                        // TODO:
                        // Warning removing against Public or Controllers.
                    }
                    _ => (),
                }
            }
            for viewer in removed {
                let principal = Principal::from_text(viewer).unwrap();
                if let Some(idx) = viewers.iter().position(|x| *x == principal) {
                    viewers.swap_remove(idx);
                }
            }
        }

        // If no viewer in the list, e.g. all removed.
        if viewers.len() == 0 {
            if let Some(visibility) = &current_visibility {
                return match visibility {
                    LogVisibility::Public => Ok(LogVisibility::Public),
                    _ => Ok(LogVisibility::Controllers),
                };
            }

            return Ok(LogVisibility::Controllers);
        }

        return Ok(LogVisibility::AllowedViewers(viewers));
    }
}
