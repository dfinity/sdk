use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_utils::interfaces::wallet::AddressEntry;

/// Print wallet's address book.
#[derive(Clap)]
pub struct AddressesOpts {}

pub async fn exec(env: &dyn Environment, _opts: AddressesOpts) -> DfxResult {
    let (entries,): (Vec<AddressEntry>,) = do_wallet_call(env, "list_addresses", (), true).await?;
    for entry in entries {
        let name = entry.name.unwrap_or_else(|| "No name set.".to_string());
        println!(
            "Id: {}, Kind: {:?}, Role: {:?}, Name: {}",
            entry.id, entry.kind, entry.role, name
        );
    }
    Ok(())
}
