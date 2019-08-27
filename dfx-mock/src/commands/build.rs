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

    let default_address = config.get_config().get_server().get_address("127.0.0.1".to_owned());
    let address = args.value_of("address").unwrap_or(default_address.as_str());

    let nodes = match args.value_of("nodes") {
        Some(n) => n.parse::<u64>()?,
        None => config.get_config().get_server().get_nodes(2),
    };
    let port = match args.value_of("port") {
        Some(port) => port.parse::<u16>()?,
        None => config.get_config().get_server().get_port(4200),
    };

    let mut fp = FakeProgress::new();
    fp.add_with_len(
        100,
        1000..4000,
        move |bar| {
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise:.green}] [{percent:>3.blue.bold}%] {msg}")
            );
            bar.set_message(format!("Starting local DFINITY network with {} node(s)...", nodes.clone()).as_str());
        },
        move |bar| {
            bar.finish_with_message(
                format!("Starting local DFINITY network with {} node(s)... Done.", nodes).as_str(),
            );
        },
    );

    fp.join();

    let addr = format!("{}:{}", address, port);
    println!("Listening for requests at {}", style(format!("http://{}", addr)).blue().bold().underlined());
    gotham::start(addr, router());

    Ok(())
}
