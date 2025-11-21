use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::MAINNET_LEDGER_CANISTER_ID;
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::operations::ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use anyhow::{Context, anyhow};
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1;
use std::str::FromStr;

/// Prints the account balance of the user
#[derive(Parser)]
pub struct BalanceOpts {
    /// Specifies an AccountIdentifier to get the balance of
    of: Option<String>,

    /// Specifies a principal to get the balance of
    #[arg(long, conflicts_with("of"))]
    of_principal: Option<Principal>,

    /// Subaccount of the selected identity to get the balance of
    #[arg(long, conflicts_with("of"))]
    subaccount: Option<Subaccount>,

    /// Canister ID of the ledger canister.
    #[arg(long)]
    ledger_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: BalanceOpts) -> DfxResult {
    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let balance = if let Some(of) = opts.of {
        let account_id = AccountIdentifier::from_str(&of)
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!(
                    "Failed to parse transfer destination from string '{}'.",
                    &of
                )
            })?;

        ledger::balance(agent, &account_id, opts.ledger_canister_id).await?
    } else {
        let canister_id = opts
            .ledger_canister_id
            .unwrap_or(MAINNET_LEDGER_CANISTER_ID);

        let owner = opts.of_principal.unwrap_or_else(|| {
            env.get_selected_identity_principal()
                .expect("Selected identity not instantiated.")
        });
        let of = icrc1::account::Account {
            owner,
            subaccount: opts.subaccount.map(|s| s.0),
        };

        let balance = ledger::icrc1_balance(agent, &canister_id, of).await?;
        ICPTs::from_e8s(balance.try_into()?)
    };

    println!("{balance}");

    Ok(())
}
