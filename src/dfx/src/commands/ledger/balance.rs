use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{AccountBalanceArgs, MAINNET_LEDGER_CANISTER_ID};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::nns_types::account_identifier::AccountIdentifier;
use crate::lib::nns_types::icpts::ICPTs;

use anyhow::{anyhow, Context};
use candid::{Decode, Encode};
use clap::Clap;
use ic_types::Principal;
use std::str::FromStr;

const ACCOUNT_BALANCE_METHOD: &str = "account_balance_dfx";

/// Prints the account balance of the user
#[derive(Clap)]
pub struct BalanceOpts {
    /// Specifies an AccountIdentifier to get the balance of
    of: Option<String>,

    /// Canister ID of the ledger canister.
    #[clap(long, value_name = "CANISTER_ID")]
    ledger_principal: Option<Principal>,

    /// Alias of the ledger canister.
    #[clap(long, value_name = "ALIAS", conflicts_with("ledger-principal"))]
    ledger_alias: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: BalanceOpts) -> DfxResult {
    let sender = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");
    let acc_id = opts
        .of
        .map_or_else(
            || Ok(AccountIdentifier::new(sender, None)),
            |v| AccountIdentifier::from_str(&v),
        )
        .map_err(|err| anyhow!(err))?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let canister_id = if let Some(principal) = opts.ledger_principal {
        principal
    } else if let Some(alias) = opts.ledger_alias {
        let canister_id_store = CanisterIdStore::for_env(env)?;
        canister_id_store.get(&alias)?
    } else {
        MAINNET_LEDGER_CANISTER_ID
    };

    let result = agent
        .query(&canister_id, ACCOUNT_BALANCE_METHOD)
        .with_arg(Encode!(&AccountBalanceArgs {
            account: acc_id.to_string()
        })?)
        .call()
        .await
        .context("Ledger account_balance call failed")?;

    let balance = Decode!(&result, ICPTs).context("Failed to decode ledger result")?;

    println!("{}", balance);

    Ok(())
}
