use crate::config::cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("show").about(UserMessage::CacheShow.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let v = format!("{}", env.get_version());
    println!("{:?}", cache::get_bin_cache(&v)?);
    Ok(())
}
