use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::AccountIdentifier;

use clap::Clap;

/// Prints the selected identity's AccountIdentifier.
#[derive(Clap)]
pub struct AccountIdOpts {}

pub async fn exec(env: &dyn Environment, _opts: AccountIdOpts) -> DfxResult {
    let sender = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");
    println!("{}", AccountIdentifier::new(sender, None));
    Ok(())
}
