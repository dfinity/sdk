use crate::config::{cache, dfx_version};
use crate::lib::env::VersionEnv;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};
use std::io::Write;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("show").about(UserMessage::CacheShow.to_str())
}

pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: VersionEnv,
{
    println!("{}", cache::get_bin_cache(env.get_version())?);
    Ok(())
}
