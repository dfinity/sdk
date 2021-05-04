use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::AccountIdentifier;

use clap::Clap;

/// Prints the selected identity's AccountIdentifier.
#[derive(Clap)]
pub struct AccountIdOpts {
	principal: Option<String>
}

pub async fn exec(env: &dyn Environment, opts: AccountIdOpts) -> DfxResult {
    if let Some(p) = opts.principal {
    	let principal = ic_types::Principal::from_text(p)?;
    	println!("{}", AccountIdentifier::new(principal, None));
    } else {
	    let sender = env
	        .get_selected_identity_principal()
	        .expect("Selected identity not instantiated.");
	    println!("{}", AccountIdentifier::new(sender, None));
    }
    Ok(())
}
