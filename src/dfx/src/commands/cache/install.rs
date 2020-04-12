use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches};

pub fn construct() -> App<'static> {
    App::new("install").about(UserMessage::CacheUnpack.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    env.get_cache().force_install()
}
