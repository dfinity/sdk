use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;

pub fn construct() -> App<'static> {
    SubCommand::with_name("new")
        .about(UserMessage::NewIdentity.to_str())
        .arg(
            Arg::new("identity")
                //.help("The identity to create.")
                .required(true)
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let name = args.value_of("identity").unwrap();

    let log = env.get_logger();
    info!(log, r#"Creating identity: "{}"."#, name);

    IdentityManager::new(env)?.create_new_identity(name)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}
