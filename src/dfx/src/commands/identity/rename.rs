use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("rename")
        .about(UserMessage::RenameIdentity.to_str())
        .arg(
            Arg::with_name("from")
                .help("The current name of the identity.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .help("The new name of the identity.")
                .required(true)
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let from = args.value_of("from").unwrap();
    let to = args.value_of("to").unwrap();

    let log = env.get_logger();
    info!(log, r#"Renaming identity "{}" to "{}"."#, from, to);

    let renamed_default = IdentityManager::new(env)?.rename(from, to)?;

    info!(log, r#"Renamed identity "{}" to "{}"."#, from, to);
    if renamed_default {
        info!(log, r#"Now using identity: "{}"."#, to);
    }

    Ok(())
}
