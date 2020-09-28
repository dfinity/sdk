use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("remove")
        .about(UserMessage::RemoveIdentity.to_str())
        .arg(
            Arg::with_name("identity")
                .help("The identity to remove.")
                .required(true)
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let name = args.value_of("identity").unwrap();

    let log = env.get_logger();
    info!(log, r#"Removing identity "{}"."#, name);

    IdentityManager::new(env)?.remove(name)?;

    info!(log, r#"Removed identity "{}"."#, name);
    Ok(())
}
