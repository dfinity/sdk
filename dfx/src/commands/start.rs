use crate::config::cache::binary_command;
use crate::config::dfinity::{Config, ConfigCanistersCanister};
use crate::lib::api_client::{
    install_code, ping, Blob, CanisterInstallCodeCall, Client, ClientConfig,
};
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

    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Starting up the DFINITY client...");
    b.enable_steady_tick(80);

    let mut cmd = binary_command(&config, "client").unwrap();
    let mut child = cmd.spawn()?;

    // Count 600 msec to give the user the impression that something is working hard.
    std::thread::sleep(std::time::Duration::from_millis(400));
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

    // Wait for the server to actually be up.
    b.set_message("Pinging the DFINITY client...");
    std::thread::sleep(std::time::Duration::from_millis(400));
    let url = format!("http://{}:{}", address, port);
    let client = Client::new(ClientConfig { url: url.clone() });

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    while !runtime.block_on(ping(client.clone())).is_ok() {}

    b.finish_with_message("DFINITY client started...");

    let addr = format!("{}:{}", address, port);
    eprintln!(
        "Listening for requests at {}\n\n",
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

                let p1 = input_as_path.clone();
                let p2 = input_as_path.clone();
                let p3 = input_as_path.clone();
                let b1 = Arc::clone(&bar);
                let b2 = Arc::clone(&bar);
                let b3 = Arc::clone(&bar);

                let url = url.clone();
                let canister_id = v.canister_id.unwrap_or(42);

                watch_file(
                    Box::new(move |name| {
                        binary_command(config.as_ref(), name).map_err(DfxError::StdIo)
                    }),
                    &input_as_path,
                    &output_root.join(x.as_str()),
                    Box::new(move || {
                        b1.set_message(format!("{} - Building...", p1.to_str().unwrap()).as_str());
                        b1.enable_steady_tick(80);
                    }),
                    Box::new(move |wasm_path| {
                        b2.set_message(format!("{} - Uploading...", p2.to_str().unwrap()).as_str());
                        let wasm = std::fs::read(wasm_path).unwrap();
                        let client = Client::new(ClientConfig {
                            url: url.to_string(),
                        });

                        let install = install_code(
                            client,
                            CanisterInstallCodeCall {
                                canister_id,
                                module: Blob(wasm),
                            },
                        );

                        let mut runtime = Runtime::new().expect("Unable to create a runtime");
                        runtime.block_on(install).unwrap();
                        b2.set_message(format!("{} - Done", p2.to_str().unwrap()).as_str());
                        b2.disable_steady_tick();
                        b2.set_position(16);
                    }),
                    Box::new(move || {
                        b3.set_message(format!("{} - Error", p3.to_str().unwrap()).as_str());
                        b3.disable_steady_tick();
                        b3.set_position(16);
                    }),
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
