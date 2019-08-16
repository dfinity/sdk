use crate::commands::CliResult;
use crate::util::fake_command;
use clap::{ArgMatches, SubCommand, Arg, App};
use std::thread;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use crate::config::Config;


pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start the local test network in the background.")
        .arg(
            Arg::with_name("port")
                .help("The port the test net API server should listen to.")
                .long("port")
                .short("p")
        )
        .arg(
            Arg::with_name("nodes")
                .help("The number of nodes to start locally. By default uses 2.")
                .long("nodes")
                .short("n")
        )
}

pub fn exec(args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let config = Config::from_current_dir()?;

    let nodes = 0; // args.value_of("nodes").into().unwrap_or(config.get_value()["nodes"].as_u64().unwrap());

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
                thread::sleep(Duration::from_millis(100));
            }
            bar.inc(1)
        }
        bar.finish_with_message("Starting local DFINITY network with 2 node(s)... Done.");



        Ok(())
    })
}
