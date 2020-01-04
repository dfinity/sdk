use crate::lib::env::{BinaryCacheEnv, VersionEnv};
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("delete")
        .about(UserMessage::CacheDelete.to_str())
        .arg(Arg::with_name("version").takes_value(true))
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv + VersionEnv,
{
    match args.value_of("version") {
        Some(v) => env.delete(v),
        _ => env.delete(env.get_version()),
    }
}
