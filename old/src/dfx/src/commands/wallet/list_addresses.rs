use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use ic_utils::interfaces::wallet::AddressEntry;

/// Print wallet's address book.
#[derive(Parser)]
pub struct AddressesOpts {}

pub async fn exec(env: &dyn Environment, _opts: AddressesOpts) -> DfxResult {
    let (entries,): (Vec<AddressEntry>,) = wallet_query(env, "list_addresses", ()).await?;
    for entry in entries {
        let name = entry.name.unwrap_or_else(|| "No name set.".to_string());
        println!(
            "Id: {}, Kind: {:?}, Role: {:?}, Name: {}",
            entry.id, entry.kind, entry.role, name
        );
    }
    Ok(())
}
