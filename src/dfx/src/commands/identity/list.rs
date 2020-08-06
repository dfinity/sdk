use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("list").about(UserMessage::ListIdentities.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let identities = IdentityManager::new(env)?.get_identity_names()?;
    for identity in identities {
        println!("{}", identity);
    }
    Ok(())
}
