use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("whoami").about(UserMessage::ShowIdentity.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let mgr = IdentityManager::new(env)?;
    let identity = mgr.get_selected_identity_name();
    println!("{}", identity);
    Ok(())
}
