use crate::config::cache::binary_command;
use crate::config::dfinity::Config;
use crate::lib::error::DfxResult;
use crate::util::FakeProgress;
use clap::{App, Arg, ArgMatches, SubCommand};
use console::style;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use hyper::http::Method;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

const HELLO_WORLD: &str = "Hello World!";

pub fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_WORLD)
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .request(vec![Method::GET, Method::HEAD], "/")
            .to(say_hello);
    })
}

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start the local test network in the background.")
        .arg(
            Arg::with_name("address")
                .help("The address to listen to. Default to 127.0.0.1 (localhost).")
                .long("address")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .help("The port the server should listen to.")
                .long("port")
                .short("p")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("nodes")
                .help("The number of nodes to start locally. By default uses 1 node.")
                .long("nodes")
                .short("n")
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

    let nodes = match args.value_of("nodes") {
        Some(n) => n.parse::<u64>()?,
        None => config.get_config().get_defaults().get_start().get_nodes(1),
    };
    let port = match args.value_of("port") {
        Some(port) => port.parse::<u16>()?,
        None => config
            .get_config()
            .get_defaults()
            .get_start()
            .get_port(8080),
    };

    let mp = MultiProgress::new();
    let b = mp.add(ProgressBar::new_spinner());

    b.set_message("Starting up the DFINITY client...");
    let mut cmd = binary_command(&config, "client").unwrap();
    let _child = cmd.spawn()?;

    // Count 600 msec to give the user the impression that something is working hard.
    for _ in 1..6 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        b.inc(1);
    }

    // Wait for the server to actually be up.
    loop {

    }
    b.finish_with_message("DFINITY client started...");
    mp.join();

    let addr = format!("{}:{}", address, port);
    println!(
        "Listening for requests at {}",
        style(format!("http://{}", addr)).blue().bold().underlined()
    );
    gotham::start(addr, router());

    Ok(())
}
