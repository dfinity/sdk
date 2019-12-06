use crate::lib::env::BinaryCacheEnv;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install").about(UserMessage::CacheUnpack.to_str())
}

pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv,
{
    env.force_install()
}
