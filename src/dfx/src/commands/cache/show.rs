use crate::config::cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches};

pub fn construct() -> App<'static> {
    App::new("show").about(UserMessage::CacheShow.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    let v = format!("{}", env.get_version());
    println!("{}", cache::get_bin_cache(&v)?.as_path().display());
    Ok(())
}
