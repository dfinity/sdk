use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};
use std::io::Write;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("list").about(UserMessage::ListIdentities.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let mgr = IdentityManager::new(env)?;
    let identities = mgr.get_identity_names()?;
    let current_identity = mgr.get_selected_identity_name();
    for identity in identities {
        if current_identity == &identity {
            // same identity, suffix with '*'.
            print!("{}", identity);
            std::io::stdout().flush()?;
            eprint!(" *");
            std::io::stderr().flush()?;
            println!();
        } else {
            println!("{}", identity);
        }
    }
    Ok(())
}
