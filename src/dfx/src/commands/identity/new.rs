use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("new")
        .about(UserMessage::NewIdentity.to_str())
        .arg(
            Arg::with_name("identity")
                .help("The identity to create.")
                .required(true)
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let name = args.value_of("identity").unwrap();

    let log = env.get_logger();
    info!(log, r#"Creating identity: "{}"."#, name);

    IdentityManager::new(env)?.create_new_identity(name)
}
