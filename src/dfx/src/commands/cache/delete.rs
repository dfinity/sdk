use crate::config::cache::delete_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static> {
    SubCommand::with_name("delete")
        .about(UserMessage::CacheDelete.to_str())
        .arg(Arg::with_name("version").takes_value(true))
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    match args.value_of("version") {
        Some(v) => delete_version(v).map(|_| {}),
        _ => env.get_cache().delete(),
    }
}
