use crate::lib::env::BinaryCacheEnv;
use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start a local network in the background.")
        .arg(
            Arg::with_name("address")
                .help("The address to listen to. Defaults to 127.0.0.1 (localhost).")
                .long("address")
                .takes_value(true),
        )
}

/// Find the binary path for the client, then start the node manager.
pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv,
{
    println!("Starting up the DFINITY node manager...");

    let client_pathbuf = env.get_binary_command_path("client")?;
    let client = client_pathbuf.as_path();

    let mut cmd = env.get_binary_command("nodemanager")?;
    cmd.args(&[client]);

    let _child = cmd.spawn()?;

    println!("DFINITY node manager started...");

    Ok(())
}
