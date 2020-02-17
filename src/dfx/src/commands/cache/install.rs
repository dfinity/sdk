use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install").about(UserMessage::CacheUnpack.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    env.get_cache().force_install()
}
