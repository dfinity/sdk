use crate::lib::error::{DfxError, DfxResult};
use crate::lib::proxy::{CoordinateProxy, Proxy, ProxyConfig};

use crossbeam::channel::{Receiver, Sender};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use indicatif::ProgressBar;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::time::Duration;

pub fn spawn_and_update_proxy(
    proxy_config: ProxyConfig,
    client_port_path: PathBuf,
    proxy_supervisor: CoordinateProxy,
    b: ProgressBar,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("Proxy".into())
        .spawn(move || {
            let proxy = Proxy::new(proxy_config);
            // Start the proxy first. Below, we panic to propagate the error
            // to the parent thread as an error via join().

            // Check the port and then start the proxy. Below, we panic to propagate the error
            // to the parent thread as an error via join().
            b.set_message("Checking client!");
            let port = retrieve_client_port(
                &client_port_path,
                proxy_supervisor.is_killed.clone(),
                proxy_supervisor.request_stop_echo.clone(),
                &b,
            )
            .unwrap_or_else(|e| {
                proxy_supervisor
                    .request_stop_echo
                    .try_send(())
                    .expect("Client thread couldn't signal parent to stop");

                panic!("Failed to watch port configuration file {:?}", e);
            });

            let proxy = proxy.set_client_api_port(port.clone());
            b.set_message(format!("Client bound at {}", port).as_str());
            proxy
                .restart(
                    proxy_supervisor.inform_parent.clone(),
                    proxy_supervisor.server_receiver.clone(),
                )
                .unwrap_or_else(|e| {
                    proxy_supervisor
                        .request_stop_echo
                        .try_send(())
                        .expect("Client thread couldn't signal parent to stop");
                    panic!("Failed to restart the proxy {:?}", e);
                });

            while proxy_supervisor.is_killed.is_empty() {
                //wait!
            }
        })
}

fn retrieve_client_port(
    client_port_path: &PathBuf,
    is_killed: Receiver<()>,
    request_stop_echo: Sender<()>,
    b: &ProgressBar,
) -> DfxResult<u16> {
    let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(100)).map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed to create watcher for port pid file: {}", e),
        ))
    })?;

    watcher
        .watch(&client_port_path, move |event| {
            if let Ok(e) = is_killed.try_recv() {
                // We are in a weird state where the replica exited with an error,
                // but we are still waiting for the pid file to change. As this change
                // is never going to occur we need to exit our wait and stop tracking
                // the file. We need to re-send the error to properly handle it later
                // on. Worst case we will panic at this point.
                #[allow(clippy::unit_arg)]
                request_stop_echo
                    // We are re-sending the signal here. It is a unit
                    // right now but that can easily change.
                    .send(e)
                    .expect("Watcher could not re-signal request to stop.");
                return Flow::Exit;
            }
            match event {
                // We pretty much want to unblock for any events
                // except a rescan. A move, create etc event should
                // lead to a failure.
                Event::Rescan => Flow::Continue,
                _ => Flow::Exit,
            }
        })
        .map_err(|e| {
            DfxError::RuntimeError(Error::new(
                ErrorKind::Other,
                format!("Failed to watch port pid file: {}", e),
            ))
        })?;
    b.set_message("Waiting for client to bind their http server port...");
    // We are blocking here and actually processing write events.

    let port_after_enter = fs::read_to_string(&client_port_path).map_err(DfxError::RuntimeError)?;

    if port_after_enter != "" {
        let port = port_after_enter
            .parse::<u16>()
            .map_err(DfxError::CouldNotParsePort);
        return port;
    }

    watcher.run();
    fs::read_to_string(&client_port_path)
        .map_err(DfxError::RuntimeError)?
        .parse::<u16>()
        .map_err(DfxError::CouldNotParsePort)
}
