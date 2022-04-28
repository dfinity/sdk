use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use anyhow::{anyhow, Context};
use ic_types::Principal;
use std::convert::TryFrom;

use clap::Parser;

/// Prints the ledger account identifier corresponding to a principal.
#[derive(Parser)]
pub struct AccountIdOpts {
    #[clap(long, value_name = "PRINCIPAL")]
    /// Principal controlling the account.
    pub of_principal: Option<Principal>,

    #[clap(long, value_name = "ALIAS")]
    /// Alias of the canister controlling the account.
    pub of_canister: Option<String>,

    #[clap(long, value_name = "SUBACCOUNT")]
    /// Subaccount identifier (64 character long hex string).
    pub subaccount: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: AccountIdOpts) -> DfxResult {
    let subaccount = match opts.subaccount {
        Some(sub_hex) => {
            let sub_bytes = hex::decode(&sub_hex)
                .with_context(|| format!("Subaccount '{}' is not a valid hex string", sub_hex))?;
            let sub = Subaccount::try_from(&sub_bytes[..])
                .with_context(|| format!("Subaccount '{}' is not 64 characters long", sub_hex))?;
            Some(sub)
        }
        None => None,
    };
    let principal = if let Some(principal) = opts.of_principal {
        if opts.of_canister.is_some() {
            return Err(anyhow!(
                "You can specify at most one of of-principal and of-canister arguments."
            ));
        }
        principal
    } else if let Some(alias) = opts.of_canister {
        let canister_id_store =
            CanisterIdStore::for_env(env).context("Failed to load canister id store")?;
        canister_id_store
            .get(&alias)
            .context(format!("Failed to get canister id for {}.", alias))?
    } else {
        env.get_selected_identity_principal()
            .context("No identity is selected")?
    };
    println!("{}", AccountIdentifier::new(principal, subaccount));
    Ok(())
}
