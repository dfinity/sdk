use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use anyhow::{anyhow, Context};
use candid::Principal;

use clap::Parser;

/// Prints the ledger account identifier corresponding to a principal.
#[derive(Parser)]
pub struct AccountIdOpts {
    #[clap(long, value_name = "PRINCIPAL")]
    /// Principal controlling the account.
    pub of_principal: Option<Principal>,

    #[clap(long, value_name = "ALIAS")]
    /// Alias or principal of the canister controlling the account.
    pub of_canister: Option<String>,

    #[clap(long, value_name = "SUBACCOUNT")]
    /// Subaccount identifier (64 character long hex string).
    pub subaccount: Option<Subaccount>,
}

pub async fn exec(env: &dyn Environment, opts: AccountIdOpts) -> DfxResult {
    let principal = if let Some(principal) = opts.of_principal {
        if opts.of_canister.is_some() {
            return Err(anyhow!(
                "You can specify at most one of of-principal and of-canister arguments."
            ));
        }
        principal
    } else if let Some(alias) = opts.of_canister {
        let canister_id_store = CanisterIdStore::for_env(env)?;
        Principal::from_text(&alias).or_else(|_| canister_id_store.get(&alias))?
    } else {
        env.get_selected_identity_principal()
            .context("No identity is selected")?
    };
    println!("{}", AccountIdentifier::new(principal, opts.subaccount));
    Ok(())
}
