use crate::commands::CliResult;
use crate::config::Config;
use crate::util::fake_command;
use clap::{ArgMatches, SubCommand, Arg, App};
use console::style;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::State;
use hyper::http::Method;
use std::thread;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};

const HELLO_WORLD: &'static str = "Hello World!";

pub fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_WORLD)
}

fn router() -> Router {
    build_simple_router(|route| {
        route.request(vec![Method::GET, Method::HEAD], "/").to(say_hello);
//        route.get_or_head("/products").to(say_hello);
//
//        route.scope("/checkout", |route| {
//            route.get("/start").to(say_hello);
//
//            // Associations allow a single path to be matched for multiple HTTP verbs
//            // with each delegating to a unique handler or the same handler, as shown here with
//            // put and patch.
//            route.associate("/address", |assoc| {
//                assoc.post().to(say_hello);
//                assoc.put().to(say_hello);
//                assoc.patch().to(say_hello);
//                assoc.delete().to(say_hello);
//            });
//
//            route
//                .post("/payment_details")
//                .to(say_hello);
//
//            route
//                .put("/payment_details")
//                .to(say_hello);
//
//            route.post("/complete").to(say_hello);
//        });
//
//        route.scope("/api", |route| {
//            route.get("/products").to(say_hello);
//        });
    })
}


pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start the local test network in the background.")
        .arg(
            Arg::with_name("address")
                .help("The address to listen to. Default to 127.0.0.1 (localhost).")
                .long("address")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("port")
                .help("The port the test net API server should listen to.")
                .long("port")
                .short("p")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("nodes")
                .help("The number of nodes to start locally. By default uses 2.")
                .long("nodes")
                .short("n")
                .takes_value(true)
        )
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

    fake_command(|| {
        let bar = ProgressBar::new(100);
        bar.set_draw_target(ProgressDrawTarget::stderr());
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("[{elapsed_precise:.green}] [{percent:>3.blue.bold}%] {msg}")
        );
        bar.set_message(format!("Starting local DFINITY network with {} node(s)...", nodes).as_str());

        for _x in 0..100 {
            if rand::random() {
                thread::sleep(Duration::from_millis(80));
            }
            bar.inc(1)
        }
        bar.finish_with_message(
            format!("Starting local DFINITY network with {} node(s)... Done.", nodes).as_str(),
        );

        let addr = format!("{}:{}", address, port);
        println!("Listening for requests at {}", style(format!("http://{}", addr)).blue().bold().underlined());
        gotham::start(addr, router());

        Ok(())
    })
}
