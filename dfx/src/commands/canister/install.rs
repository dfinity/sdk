use crate::lib::env::{ClientEnv, ProjectConfigEnv};
use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install")
        .about("Install a canister. Will build it")
        .arg(Arg::with_name("canister").help("The canister name to build."))
}

pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv + ProjectConfigEnv,
{
    // Read the config.
    let _config = env.get_config().unwrap();

    println!("INSTALL!");

    Ok(())
}
