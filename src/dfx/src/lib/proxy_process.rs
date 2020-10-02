use crate::lib::error::DfxError;
use crate::lib::proxy::{CoordinateProxy, Proxy, ProxyConfig};

use crossbeam::unbounded;
use futures::executor::block_on;
use hotwatch::{Event, Hotwatch};
use indicatif::ProgressBar;
use std::fs;
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
            b.set_message("Checking replica!");

            let (send_port, rcv_port) = unbounded();

            let mut hotwatch = Hotwatch::new_with_custom_delay(std::time::Duration::from_secs(1))
                .expect("hotwatch failed to initialize!");
            let is_killed = proxy_supervisor.is_killed.clone();
            // We start a hotwatch watcher. It will run on the
            // background, with "watch life" equal to the lifetime of
            // the value. We attempt on each sensible event to read
            // the port.
            hotwatch
                .watch(&client_port_path, {
                    let client_port_path = client_port_path.clone();
                    move |event: Event| {
                        if !is_killed.is_empty() {
                            // We are in a weird state where the replica exited with an error,
                            // but we are still waiting for the pid file to change. As this change
                            // is never going to occur we need to exit our wait and stop tracking
                            // the file. We need to re-send the error to properly handle it later
                            // on. Worst case we will panic at this point.
                            //
                            // Disconnect the sender. This should unblock the port receiver.
                            let _ = send_port;
                        }
                        match event {
                            // We pretty much want to unblock for any events
                            // except a rescan. A move, create etc event should
                            // lead to a failure.
                            Event::Rescan => {}
                            _ => {
                                let port = fs::read_to_string(&client_port_path)
                                    .map_err(DfxError::RuntimeError)
                                    .expect("failed to read port file")
                                    .parse::<u16>();
                                if let Ok(port) = port {
                                    send_port.send(port).expect("Failed to send port");
                                }
                            }
                        }
                    }
                })
                .expect("failed to watch replica port file!");

            let port_after_enter =
                fs::read_to_string(&client_port_path).unwrap_or_else(|_| "".to_owned());

            let port = if port_after_enter != "" {
                port_after_enter
                    .parse::<u16>()
                    .map_err(DfxError::CouldNotParsePort)
            } else {
                // Fail if sender is disconnected.
                Ok(rcv_port.recv().expect("Failed to receive port"))
            }
            .unwrap_or_else(|e| {
                proxy_supervisor
                    .request_stop_echo
                    .try_send(())
                    .expect("Replica thread couldn't signal parent to stop");

                panic!("Failed to watch port configuration file {:?}", e);
            });
            // Stop watching.
            let _ = hotwatch;
            let proxy = proxy.set_client_api_port(port);
            b.set_message(format!("Replica bound at {}", port).as_str());
            block_on(proxy.restart(
                proxy_supervisor.inform_parent.clone(),
                proxy_supervisor.server_receiver.clone(),
            ))
            .unwrap_or_else(|e| {
                proxy_supervisor
                    .request_stop_echo
                    .try_send(())
                    .expect("Replica thread couldn't signal parent to stop");
                panic!("Failed to restart the proxy {:?}", e);
            });

            while proxy_supervisor.is_killed.is_empty() {
                std::thread::sleep(Duration::from_millis(1000));
                //wait!
            }
        })
}
