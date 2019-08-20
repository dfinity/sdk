use crate::commands::CliResult;
use crate::config::Config;
use crate::util::FakeProgress;
use clap::{ArgMatches, SubCommand, Arg, App};
use console::style;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::State;
use hyper::http::Method;
use indicatif::ProgressStyle;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Start the local test network in the background.")
}

pub fn exec(args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let config = Config::from_current_dir()?;



    Ok(())
}
