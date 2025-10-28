use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::operations::canister::{
    get_canister_status, list_canister_snapshots, update_settings,
};
use crate::lib::operations::migration_canister::{
    MigrationStatus, NNS_MIGRATION_CANISTER_ID, migrate_canister, migrate_status,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::subnet::get_subnet_for_canister;
use crate::util::ask_for_consent;
use anyhow::{Context, bail};
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_management_canister_types::CanisterStatusType;
use num_traits::ToPrimitive;
use slog::{debug, error, info};
use std::time::Duration;
use time::{OffsetDateTime, macros::format_description};

/// Renames a canister.
#[derive(Parser)]
#[command(override_usage = "dfx canister rename [OPTIONS] <FROM_CANISTER> --rename-to <RENAME_TO>")]
pub struct CanisterRenameOpts {
    /// Specifies the name or id of the canister to rename.
    from_canister: String,

    /// Specifies the name or id of the canister to rename to.
    #[arg(long)]
    rename_to: String,

    /// Skips yes/no checks by answering 'yes'. Not recommended outside of CI.
    #[arg(long, short)]
    yes: bool,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterRenameOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let log = env.get_logger();
    let agent = env.get_agent();
    let canister_id_store = env.get_canister_id_store()?;

    // Get the canister IDs.
    let from_canister = opts.from_canister.as_str();
    let to_canister = opts.rename_to.as_str();
    let from_canister_id =
        Principal::from_text(from_canister).or_else(|_| canister_id_store.get(from_canister))?;
    let to_canister_id =
        Principal::from_text(to_canister).or_else(|_| canister_id_store.get(to_canister))?;

    if from_canister_id == to_canister_id {
        bail!("From and rename_to canister IDs are the same");
    }

    if !opts.yes {
        ask_for_consent(
            env,
            &format!(
                "The from canister '{from_canister}' will be removed from its own subnet. Continue anyway?",
            ),
        )?;
    }

    let from_status = get_canister_status(env, from_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {from_canister}"))?;
    let to_status = get_canister_status(env, to_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {to_canister}"))?;

    ensure_canister_stopped(from_status.status, from_canister)?;
    ensure_canister_stopped(to_status.status, to_canister)?;

    // Check the cycles balance of from_canister.
    let cycles = from_status
        .cycles
        .0
        .to_u128()
        .expect("Unable to parse cycles");
    if cycles < 5_000_000_000_000 {
        bail!("The from canister {from_canister} has less than 5T cycles");
    }
    if !opts.yes && cycles > 10_000_000_000_000 {
        ask_for_consent(
            env,
            &format!("The from canister {from_canister} has more than 10T cycles. Continue?"),
        )?;
    }

    // Check that the from canister has no snapshots.
    let from_snapshots = list_canister_snapshots(env, from_canister_id, call_sender).await?;
    if !from_snapshots.is_empty() {
        bail!("The from canister {} has snapshots", from_canister);
    }

    // Check that the two canisters are on different subnets.
    let from_subnet = get_subnet_for_canister(agent, from_canister_id).await?;
    let to_subnet = get_subnet_for_canister(agent, to_canister_id).await?;
    if from_subnet == to_subnet {
        bail!("The from and rename_to canisters are on the same subnet");
    }

    // Add the NNS migration canister as a controller to the from canister.
    let mut controllers = from_status.settings.controllers.clone();
    if !controllers.contains(&NNS_MIGRATION_CANISTER_ID) {
        controllers.push(NNS_MIGRATION_CANISTER_ID);
        let settings = CanisterSettings {
            controllers: Some(controllers),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
            log_visibility: None,
            environment_variables: None,
        };
        update_settings(env, from_canister_id, settings, call_sender).await?;
    }

    // Add the NNS migration canister as a controller to the rename_to canister.
    let mut controller_added = false;
    let mut controllers = to_status.settings.controllers.clone();
    if !controllers.contains(&NNS_MIGRATION_CANISTER_ID) {
        controllers.push(NNS_MIGRATION_CANISTER_ID);
        let settings = CanisterSettings {
            controllers: Some(controllers),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
            log_visibility: None,
            environment_variables: None,
        };
        update_settings(env, to_canister_id, settings, call_sender).await?;
        controller_added = true;
    }

    // Migrate the from canister to the rename_to canister.
    debug!(log, "Renaming {from_canister} to {to_canister}");
    migrate_canister(agent, from_canister_id, to_canister_id).await?;

    // Wait for migration to complete.
    let spinner = env.new_spinner("Waiting for renaming to complete...".into());
    loop {
        let statuses = migrate_status(agent, from_canister_id, to_canister_id).await?;
        match statuses.first() {
            Some(MigrationStatus::InProgress { status }) => {
                spinner.set_message(format!("Renaming in progress: {status}").into());
            }
            Some(MigrationStatus::Succeeded { time }) => {
                spinner.finish_and_clear();
                info!(log, "Renaming succeeded at {}", format_time(time));
                break;
            }
            Some(MigrationStatus::Failed { reason, time }) => {
                spinner.finish_and_clear();
                error!(log, "Renaming failed at {}: {}", format_time(time), reason);
                break;
            }
            None => (),
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Remove the NNS migration canister from the controllers if added.
    if controller_added {
        let controllers = to_status.settings.controllers.clone();
        let settings = CanisterSettings {
            controllers: Some(controllers),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
            log_visibility: None,
            environment_variables: None,
        };
        update_settings(env, to_canister_id, settings, call_sender).await?;
    }

    Ok(())
}

fn ensure_canister_stopped(status: CanisterStatusType, canister: &str) -> DfxResult {
    match status {
        CanisterStatusType::Stopped => Ok(()),
        CanisterStatusType::Running => {
            bail!("Canister {canister} is running. Run 'dfx canister stop' first");
        }
        CanisterStatusType::Stopping => {
            bail!("Canister {canister} is stopping. Wait a few seconds and try again");
        }
    }
}

fn format_time(time: &u64) -> String {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
    OffsetDateTime::from_unix_timestamp_nanos(*time as i128)
        .unwrap()
        .format(&format)
        .unwrap()
}
