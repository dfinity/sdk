use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::operations::canister::{
    get_canister_status, list_canister_snapshots, update_settings,
};
use crate::lib::operations::migration_canister::{
    MigrationStatus, NNS_MIGRATION_CANISTER_ID, migrate_canister, migration_status,
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

/// Migrate a canister ID from one subnet to another.
#[derive(Parser)]
#[command(override_usage = "dfx canister migrate-id [OPTIONS] <FROM_CANISTER> --replace <REPLACE>")]
pub struct CanisterMigrateIdOpts {
    /// Specifies the name or id of the canister to migrate.
    from_canister: String,

    /// Specifies the name or id of the canister to replace.
    #[arg(long)]
    replace: String,

    /// Skips yes/no checks by answering 'yes'. Not recommended outside of CI.
    #[arg(long, short)]
    yes: bool,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterMigrateIdOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let log = env.get_logger();
    let agent = env.get_agent();
    let canister_id_store = env.get_canister_id_store()?;

    // Get the canister IDs.
    let from_canister = opts.from_canister.as_str();
    let target_canister = opts.replace.as_str();
    let from_canister_id =
        Principal::from_text(from_canister).or_else(|_| canister_id_store.get(from_canister))?;
    let target_canister_id = Principal::from_text(target_canister)
        .or_else(|_| canister_id_store.get(target_canister))?;

    if from_canister_id == target_canister_id {
        bail!("From and target canister IDs are the same");
    }

    if !opts.yes {
        ask_for_consent(
            env,
            &format!(
                "The target canister '{target_canister}' will be removed from its own subnet. Continue anyway?",
            ),
        )?;
    }

    let from_status = get_canister_status(env, from_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {from_canister}"))?;
    let target_status = get_canister_status(env, target_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {target_canister}"))?;

    ensure_canister_stopped(from_status.status, from_canister)?;
    ensure_canister_stopped(target_status.status, target_canister)?;

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

    // Check that the target canister has no snapshots.
    let snapshots = list_canister_snapshots(env, target_canister_id, call_sender).await?;
    if !snapshots.is_empty() {
        bail!("The target canister {} has snapshots", target_canister);
    }

    // Check that the two canisters are on different subnets.
    let from_subnet = get_subnet_for_canister(agent, from_canister_id).await?;
    let target_subnet = get_subnet_for_canister(agent, target_canister_id).await?;
    if from_subnet == target_subnet {
        bail!("The from and target canisters are on the same subnet");
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
    let mut controllers = target_status.settings.controllers.clone();
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
        update_settings(env, target_canister_id, settings, call_sender).await?;
        controller_added = true;
    }

    // Migrate the from canister to the rename_to canister.
    debug!(log, "Migrate {from_canister} to {target_canister}");
    migrate_canister(agent, from_canister_id, target_canister_id).await?;

    // Wait for migration to complete.
    let spinner = env.new_spinner("Waiting for migration to complete...".into());
    loop {
        let statuses = migration_status(agent, from_canister_id, target_canister_id).await?;
        match statuses.first() {
            Some(MigrationStatus::InProgress { status }) => {
                spinner.set_message(format!("Migration in progress: {status}").into());
            }
            Some(MigrationStatus::Succeeded { time }) => {
                spinner.finish_and_clear();
                info!(log, "Migration succeeded at {}", format_time(time));
                break;
            }
            Some(MigrationStatus::Failed { reason, time }) => {
                spinner.finish_and_clear();
                error!(log, "Migration failed at {}: {}", format_time(time), reason);
                break;
            }
            None => (),
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Remove the NNS migration canister from the controllers if added.
    if controller_added {
        let controllers = from_status.settings.controllers.clone();
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

    canister_id_store.remove(log, target_canister)?;

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
