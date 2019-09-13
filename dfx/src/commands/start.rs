use crate::lib::api_client::{ping, Client, ClientConfig};
use crate::lib::env::BinaryResolverEnv;
use crate::lib::error::{DfxError, DfxResult};
use clap::ArgMatches;
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::time::{Duration, Instant};
use tokio::prelude::FutureExt;
use tokio::runtime::Runtime;

const TIMEOUT_IN_SECS: u64 = 5;

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryResolverEnv,
{
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Starting up the client...");
    b.enable_steady_tick(80);

    let _child = {
        let client_pathbuf = env.get_binary_command_path("client")?;
        let client = client_pathbuf.as_path();

        let mut cmd = env.get_binary_command("nodemanager")?;
        cmd.args(&[client]);

        cmd.spawn()?
    };
    b.set_message("Pinging the DFINITY client...");

    std::thread::sleep(Duration::from_millis(500));

    let url = String::from(args.value_of("host").unwrap_or("http://localhost:8080"));

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    // Try to ping for 1 second, then timeout after 5 seconds if ping hasn't succeeded.
    let start = Instant::now();
    while {
        let client = Client::new(ClientConfig { url: url.clone() });

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

    Ok(())
}
