use crate::config::cache::{get_binary_path_from_config};
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
    let config = Config::from_current_dir()?;

    println!("Starting up the DFINITY node manager...");

    let client_pathbuf = get_binary_path_from_config(&config, "client")?;
    let client = client_pathbuf.as_path();

    let nodemanager = get_binary_path_from_config(&config, "nodemanager")?;

    let mut cmd = std::process::Command::new(nodemanager);
    cmd.args(&[client]);

    let _child = cmd.spawn()?;

    println!("DFINITY node manager started...");

    Ok(())
}
