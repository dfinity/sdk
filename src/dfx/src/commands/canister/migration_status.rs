use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister_migration::{MigrationStatus, migration_status};
use crate::lib::root_key::fetch_root_key_if_needed;

use candid::Principal;
use clap::Parser;
use slog::info;
use time::{OffsetDateTime, macros::format_description};

/// Show the status of a migration.
#[derive(Parser)]
#[command(
    override_usage = "dfx canister migration-status [OPTIONS] <CANISTER> --replace <REPLACE>"
)]
pub struct CanisterMigrationStatusOpts {
    /// Specifies the name or id of the canister to migrate.
    canister: String,

    /// Specifies the name or id of the canister to replace.
    #[arg(long)]
    replace: String,
}

pub async fn exec(env: &dyn Environment, opts: CanisterMigrationStatusOpts) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let log = env.get_logger();
    let agent = env.get_agent();
    let canister_id_store = env.get_canister_id_store()?;

    // Get the canister IDs.
    let migrated_canister = opts.canister.as_str();
    let replaced_canister = opts.replace.as_str();
    let migrated_canister_id = Principal::from_text(migrated_canister)
        .or_else(|_| canister_id_store.get(migrated_canister))
        .map_err(|_| {
            anyhow::anyhow!(
                "Cannot find canister '{migrated_canister}'. Please use canister id instead"
            )
        })?;
    let replaced_canister_id = Principal::from_text(replaced_canister)
        .or_else(|_| canister_id_store.get(replaced_canister))
        .map_err(|_| {
            anyhow::anyhow!(
                "Cannot find canister '{replaced_canister}'. Please use canister id instead"
            )
        })?;

    let Some(status) = migration_status(agent, migrated_canister_id, replaced_canister_id).await?
    else {
        info!(
            log,
            "No migration status found for canister '{migrated_canister}' to '{replaced_canister}'"
        );
        return Ok(());
    };

    // Print the statuses in a table with aligned columns.
    let migrated_canister_text = migrated_canister_id.to_text();
    let replaced_canister_text = replaced_canister_id.to_text();
    let status_strings: Vec<String> = vec![format_status(&status)];

    let header_migrated_canister = "Canister";
    let header_replaced_canister = "Canister To Be Replaced";
    let header_status = "Migration Status";

    let migrated_canister_width = header_migrated_canister.len().max(migrated_canister_text.len());
    let replaced_canister_width = header_replaced_canister.len().max(replaced_canister_text.len());
    let status_width = header_status
        .len()
        .max(status_strings.iter().map(|s| s.len()).max().unwrap_or(0));

    let sep_migrated_canister = "-".repeat(migrated_canister_width);
    let sep_replaced_canister = "-".repeat(replaced_canister_width);
    let sep_status = "-".repeat(status_width);

    info!(
        log,
        "| {:<s_w$} | {:<t_w$} | {:<st_w$} |",
        header_migrated_canister,
        header_replaced_canister,
        header_status,
        s_w = migrated_canister_width,
        t_w = replaced_canister_width,
        st_w = status_width
    );
    info!(
        log,
        "| {:<s_w$} | {:<t_w$} | {:<st_w$} |",
        sep_migrated_canister,
        sep_replaced_canister,
        sep_status,
        s_w = migrated_canister_width,
        t_w = replaced_canister_width,
        st_w = status_width
    );
    for status in status_strings {
        info!(
            log,
            "| {:<s_w$} | {:<t_w$} | {:<st_w$} |",
            migrated_canister_text,
            replaced_canister_text,
            status,
            s_w = migrated_canister_width,
            t_w = replaced_canister_width,
            st_w = status_width
        );
    }

    Ok(())
}

fn format_status(status: &MigrationStatus) -> String {
    match status {
        MigrationStatus::InProgress { status } => {
            format!("In progress: {status}")
        }
        MigrationStatus::Failed { reason, time } => {
            format!("Failed: {reason} at {}", format_time(time))
        }
        MigrationStatus::Succeeded { time } => {
            format!("Succeeded at {}", format_time(time))
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
