use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;

pub fn construct() -> App<'static> {
    SubCommand::with_name("use")
        .about(UserMessage::UseIdentity.to_str())
        .arg(
            Arg::new("identity")
                //.help("The identity to use.")
                .required(true)
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let identity = args.value_of("identity").unwrap();

    let log = env.get_logger();
    info!(log, r#"Using identity: "{}"."#, identity);

    IdentityManager::new(env)?.use_identity_named(identity)
}
