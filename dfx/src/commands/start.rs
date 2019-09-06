use crate::config;
use crate::config::cache::{binary_command_from_version, get_binary_path_from_version};
use crate::config::dfinity::Config;
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

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    // Read the config.
    let version: String = Config::from_current_dir().ok().map_or_else(|| config::DFX_VERSION.to_string(), |config| {
        config.get_config().get_dfx()
    });

    println!("Starting up the DFINITY node manager...");

    let client_pathbuf = get_binary_path_from_version(&version, "client")?;
    let client = client_pathbuf.as_path();

    let mut cmd = binary_command_from_version(&version, "nodemanager")?;
    cmd.args(&[client]);

    let _child = cmd.spawn()?;

    println!("DFINITY node manager started...");

    Ok(())
}
