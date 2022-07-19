use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{AccountBalanceArgs, MAINNET_LEDGER_CANISTER_ID};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;

use anyhow::{anyhow, Context};
use candid::Principal;
use candid::{Decode, Encode};
use clap::Parser;
use std::str::FromStr;

const ACCOUNT_BALANCE_METHOD: &str = "account_balance_dfx";

/// Prints the account balance of the user
#[derive(Parser)]
pub struct BalanceOpts {
    /// Specifies an AccountIdentifier to get the balance of
    of: Option<String>,

    /// Subaccount of the selected identity to get the balance of
    #[clap(long, conflicts_with("of"))]
    subaccount: Option<Subaccount>,

    /// Canister ID of the ledger canister.
    #[clap(long)]
    ledger_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: BalanceOpts) -> DfxResult {
    let sender = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");
    let subacct = opts.subaccount;
    let acc_id = opts
        .of
        .map_or_else(
            || Ok(AccountIdentifier::new(sender, subacct)),
            |v| AccountIdentifier::from_str(&v),
        )
        .map_err(|err| anyhow!(err))?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let canister_id = opts
        .ledger_canister_id
        .unwrap_or(MAINNET_LEDGER_CANISTER_ID);

    let result = agent
        .query(&canister_id, ACCOUNT_BALANCE_METHOD)
        .with_arg(
            Encode!(&AccountBalanceArgs {
                account: acc_id.to_string()
            })
            .context("Failed to encode arguments.")?,
        )
        .call()
        .await
        .with_context(|| {
            format!(
                "Failed query call to {} for method {}.",
                canister_id, ACCOUNT_BALANCE_METHOD
            )
        })?;

    let balance = Decode!(&result, ICPTs).context("Failed to decode response.")?;

    println!("{}", balance);

    Ok(())
}
