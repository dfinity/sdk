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
    let source_canister = opts.canister.as_str();
    let target_canister = opts.replace.as_str();
    let source_canister_id = Principal::from_text(source_canister)
        .or_else(|_| canister_id_store.get(source_canister))
        .map_err(|_| {
            anyhow::anyhow!(
                "Cannot find canister '{source_canister}'. Please use canister id instead"
            )
        })?;
    let target_canister_id = Principal::from_text(target_canister)
        .or_else(|_| canister_id_store.get(target_canister))
        .map_err(|_| {
            anyhow::anyhow!(
                "Cannot find canister '{target_canister}'. Please use canister id instead"
            )
        })?;

    let statuses = migration_status(agent, source_canister_id, target_canister_id).await?;

    if statuses.is_empty() {
        info!(
            log,
            "No migration status found for canister '{source_canister}' to '{target_canister}'"
        );
        return Ok(());
    }

    // Print the statuses in a table with aligned columns.
    let source_text = source_canister_id.to_text();
    let target_text = target_canister_id.to_text();
    let status_strings: Vec<String> = statuses.iter().map(format_status).collect();

    let header_source = "Canister";
    let header_target = "Canister To Be Replaced";
    let header_status = "Migration Status";

    let source_width = header_source.len().max(source_text.len());
    let target_width = header_target.len().max(target_text.len());
    let status_width = header_status
        .len()
        .max(status_strings.iter().map(|s| s.len()).max().unwrap_or(0));

    let sep_source = "-".repeat(source_width);
    let sep_target = "-".repeat(target_width);
    let sep_status = "-".repeat(status_width);

    info!(
        log,
        "| {:<s_w$} | {:<t_w$} | {:<st_w$} |",
        header_source,
        header_target,
        header_status,
        s_w = source_width,
        t_w = target_width,
        st_w = status_width
    );
    info!(
        log,
        "| {:<s_w$} | {:<t_w$} | {:<st_w$} |",
        sep_source,
        sep_target,
        sep_status,
        s_w = source_width,
        t_w = target_width,
        st_w = status_width
    );
    for status in status_strings {
        info!(
            log,
            "| {:<s_w$} | {:<t_w$} | {:<st_w$} |",
            source_text,
            target_text,
            status,
            s_w = source_width,
            t_w = target_width,
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
