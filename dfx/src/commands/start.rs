use crate::config::cache::{get_binary_path_from_config};
use crate::config::dfinity::Config;
use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};
use console::style;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start a local  network in the background.")
        .arg(
            Arg::with_name("address")
                .help("The address to listen to. Defaults to 127.0.0.1 (localhost).")
                .long("address")
                .takes_value(true),
        )
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    // Read the config.
    let config = Config::from_current_dir()?;

    let default_address = &config.get_config().get_defaults().get_start().address;
    let default_address = default_address
        .clone()
        .unwrap_or_else(|| "127.0.0.1".to_owned());
    let address = args
        .value_of("address")
        .unwrap_or_else(|| default_address.as_str());

    // let nodes = match args.value_of("nodes") {
    //     Some(n) => n.parse::<u64>()?,
    //     None => config.get_config().get_defaults().get_start().get_nodes(1),
    // };
    let port = match args.value_of("port") {
        Some(port) => port.parse::<u16>()?,
        None => config
            .get_config()
            .get_defaults()
            .get_start()
            .get_port(8080),
    };

    println!("Starting up the DFINITY node manager...");

    let client_pathbuf = get_binary_path_from_config(&config, "client")?;
    let client = client_pathbuf.as_path();

    let nodemanager = get_binary_path_from_config(&config, "nodemanager")?;

    let mut cmd = std::process::Command::new(nodemanager);
    cmd.args(&[client]);

    let _child = cmd.spawn()?;

    println!("DFINITY node manager started...");

    let addr = format!("{}:{}", address, port);
    println!(
        "Listening for requests at {}",
        style(format!("http://{}", addr)).blue().bold().underlined()
    );

    Ok(())
}
