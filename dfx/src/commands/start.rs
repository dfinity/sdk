use crate::lib::api_client::{ping, Client, ClientConfig};
use crate::lib::env::{BinaryResolverEnv, ProjectConfigEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::webserver::webserver;
use clap::{App, Arg, ArgMatches, SubCommand};
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::time::{Duration, Instant};
use tokio::prelude::FutureExt;
use tokio::runtime::Runtime;

const TIMEOUT_IN_SECS: u64 = 10;
const IC_CLIENT_BIND_ADDR: &str = "http://localhost:8080/api";

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about("Start a local network in the background.")
        .arg(
            Arg::with_name("host")
                .help("The host (with port) to bind the frontend to.")
                .long("host")
                .takes_value(true),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ProjectConfigEnv + BinaryResolverEnv,
{
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Starting up the client...");
    b.enable_steady_tick(80);

    let client_pathbuf = env.get_binary_command_path("client").unwrap();
    let nodemanager_pathbuf = env.get_binary_command_path("nodemanager").unwrap();

    let config = env.get_config().unwrap();
    let address_and_port = args
        .value_of("host")
        .and_then(|host| Option::from(host.parse()))
        .unwrap_or_else(|| {
            Ok(config
                .get_config()
                .get_defaults()
                .get_start()
                .get_binding_socket_addr("localhost:8000")
                .unwrap())
        })?;
    let frontend_url = format!(
        "http://{}:{}",
        address_and_port.ip(),
        address_and_port.port()
    );
    let project_root = config.get_path().parent().unwrap();

    let client_watchdog = std::thread::spawn(move || {
        let client = client_pathbuf.as_path();
        let nodemanager = nodemanager_pathbuf.as_path();
        loop {
            let mut cmd = std::process::Command::new(nodemanager);
            cmd.args(&[client]);
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());

            // If the nodemanager itself fails, we are probably deeper into troubles than
            // we can solve at this point and the user is better rerunning the server.
            let mut child = cmd.spawn().unwrap();
            if child.wait().is_err() {
                break;
            }
        }
    });
    let frontend_watchdog = webserver(
        address_and_port,
        url::Url::parse(IC_CLIENT_BIND_ADDR).unwrap(),
        project_root
            .join(
                config
                    .get_config()
                    .get_defaults()
                    .get_start()
                    .get_serve_root(".")
                    .as_path(),
            )
            .as_path(),
    );

    b.set_message("Pinging the DFINITY client...");

    std::thread::sleep(Duration::from_millis(500));

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    // Try to ping for 1 second, then timeout after 5 seconds if ping hasn't succeeded.
    let start = Instant::now();
    while {
        let client = Client::new(ClientConfig {
            url: frontend_url.clone(),
        });

        runtime
            .block_on(ping(client).timeout(Duration::from_millis(TIMEOUT_IN_SECS * 1000 / 4)))
            .is_err()
    } {
        if Instant::now().duration_since(start) > Duration::from_secs(TIMEOUT_IN_SECS) {
            return Err(DfxError::Unknown(
                "Timeout during start of the client.".to_owned(),
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    b.finish_with_message("DFINITY client started...");

    frontend_watchdog.join().unwrap();
    client_watchdog.join().unwrap();
    Ok(())
}
