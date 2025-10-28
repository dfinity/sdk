use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::operations::canister::{get_canister_status, stop_canister, update_settings};
use crate::lib::operations::migration_canister::{NNS_MIGRATION_CANISTER_ID, migrate_canister};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::subnet::get_subnet_for_canister;
use crate::util::ask_for_consent;
use anyhow::{Context, bail};
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use num_traits::ToPrimitive;
use slog::info;

/// Renames a canister.
#[derive(Parser)]
#[command(override_usage = "dfx canister rename <FROM_CANISTER> --rename-to <RENAME_TO>")]
pub struct CanisterRenameOpts {
    /// Specifies the name or id of the canister to rename.
    from_canister: String,

    /// Specifies the name or id of the canister to rename to.
    #[arg(long)]
    rename_to: String,
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

    // Stop both canisters.
    info!(
        log,
        "Stopping canister {}, with canister_id {}",
        from_canister,
        from_canister_id.to_text(),
    );
    stop_canister(env, from_canister_id, call_sender).await?;
    info!(
        log,
        "Stopping canister {}, with canister_id {}",
        to_canister,
        to_canister_id.to_text(),
    );
    stop_canister(env, to_canister_id, call_sender).await?;

    // Check the cycles balance of from_canister.
    let from_status = get_canister_status(env, from_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {}", from_canister))?;

    let cycles = from_status
        .cycles
        .0
        .to_u128()
        .expect("Unable to parse cycles");
    if cycles < 5_000_000_000_000 {
        bail!("Canister {} has less than 10T cycles", from_canister);
    }
    if cycles > 10_000_000_000_000 {
        ask_for_consent(
            env,
            &format!(
                "Canister {} has more than 10T cycles. Continue?",
                from_canister
            ),
        )?;
    }

    // Check if the two canisters are on different subnets.
    let from_subnet = get_subnet_for_canister(agent, from_canister_id).await?;
    let to_subnet = get_subnet_for_canister(agent, to_canister_id).await?;
    if from_subnet == to_subnet {
        bail!("From and rename_to canisters are on the same subnet");
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
    let to_status = get_canister_status(env, to_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {}", to_canister))?;
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
    }

    // Migrate the from canister to the rename_to canister.
    migrate_canister(agent, from_canister_id, to_canister_id).await?;

    Ok(())
}
