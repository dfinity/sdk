use crate::config::cache::binary_command;
use crate::config::dfinity::{Config, ConfigCanistersCanister};
use crate::lib::api_client::{ping, Client, ClientConfig};
use crate::lib::build::watch_file;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use std::io::Read;
use std::sync::Arc;
use tokio::runtime::Runtime;

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

    let _nodes = match args.value_of("nodes") {
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
    mp.set_draw_target(ProgressDrawTarget::stderr());

    let b = mp.add(ProgressBar::new_spinner());
    b.set_message("Starting up the DFINITY client...");

    let mut cmd = binary_command(&config, "client").unwrap();
    let mut child = cmd.spawn()?;
    let mut i = 0;

    // Count 600 msec to give the user the impression that something is working hard.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        b.inc(1);
        i += 1;
        if i > 5 {
            break;
        }

        // Make sure the child is still running.
        if let Ok(result) = child.try_wait() {
            if result.is_some() {
                let mut stderr = String::new();
                child.stderr.unwrap().read_to_string(&mut stderr)?;
                b.finish_with_message("The client exited early.");
                return Err(DfxError::Unknown(format!(
                    "Client could not be started.\nOutput:\n{}",
                    stderr,
                )));
            }
        }
    }

    // Wait for the server to actually be up.
    b.set_message("Pinging the DFINITY client...");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(80));
        b.inc(1);
        let client = Client::new(ClientConfig {
            url: format!("http://{}:{}", address, port),
        });

        let mut runtime = Runtime::new().expect("Unable to create a runtime");
        // TODO: not block but keep updating the spinner.
        if runtime.block_on(ping(client)).is_ok() {
            break;
        }
    }
    b.finish_with_message("DFINITY client started...");
    mp.join()?;

    let addr = format!("{}:{}", address, port);
    println!(
        "Listening for requests at {}",
        style(format!("http://{}", addr)).blue().bold().underlined()
    );

    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let output_root = project_root.join(
        config
            .get_config()
            .get_defaults()
            .get_build()
            .get_output("build/"),
    );

    if let Some(canisters) = &config.get_config().canisters {
        let config = config.clone();

        for (_, v) in canisters {
            let v: ConfigCanistersCanister = serde_json::from_value(v.to_owned())?;

            if let Some(x) = v.main {
                let input_as_path = project_root.join(x.as_str());

                let bar = Arc::new(mp.add(ProgressBar::new_spinner()));
                let config = Arc::new(config.clone());

                watch_file(
                    Box::new(move |name| {
                        binary_command(Arc::clone(&config).as_ref(), name).map_err(DfxError::StdIo)
                    }),
                    &input_as_path,
                    &output_root.join(x.as_str()),
                    Box::new(|| Arc::clone(&bar).as_ref().enable_steady_tick(80)),
                    Box::new(|| Arc::clone(&bar).as_ref().disable_steady_tick()),
                )?;
            }
        }
    }

    mp.join()?;
    loop {
        #[allow(unused_must_use)]
        {
            child.wait();
        }

        child = cmd.spawn()?;
    }
}
