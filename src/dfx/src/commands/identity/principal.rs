use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("get-principal").about(UserMessage::GetPrincipalId.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let principal_id = identity.sender()?;
    println!("{}", principal_id.to_text());
    Ok(())
}
