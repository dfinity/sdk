use crate::actors::replica_webserver_coordinator::signals::{PortReadySignal, PortReadySubscribe};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::replica_config::ReplicaConfig;

use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, Handler, Recipient,
    ResponseActFuture, Running, WrapFuture,
};
use anyhow::anyhow;
use crossbeam::channel::{unbounded, Receiver, Sender};
use delay::{Delay, Waiter};
use slog::{debug, info, Logger};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;
use std::net::SocketAddr;
use crate::actors::icx_proxy::signals::IcxProxyRestarted;

pub mod signals {
    use actix::prelude::*;

    /// A message sent to the Replica when the process is restarted. Since we're
    /// restarting inside our own actor, this message should not be exposed.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct IcxProxyRestarted {
        pub port: u16,
    }
}

pub struct IcxProxyConfig
{
    // --address 127.0.0.1:3000               (where to listen)
    pub bind: SocketAddr,

    // --replica http://localhost:8000/


}

/// The configuration for the icx_proxy actor.
pub struct Config {
    pub logger: Option<Logger>,

    pub port_ready_subscribe: Recipient<PortReadySubscribe>,
    pub shutdown_controller: Addr<ShutdownController>,

    pub icx_proxy_config: IcxProxyConfig,
    pub icx_proxy_path: PathBuf,
    pub icx_proxy_pid_path: PathBuf,
    //pub replica_configuration_dir: PathBuf,
}

/// A replica actor. Starts the replica, can subscribe to a Ready signal and a
/// Killed signal.
/// This starts a thread that monitors the process and send signals to any subscriber
/// listening for restarts. The message contains the port the replica is listening to.
///
/// Signals
///   - PortReadySubscribe
///     Subscribe a recipient (address) to receive a PortReadySignal message when
///     the replica is ready to listen to a port. The message can be sent multiple
///     times (e.g. if the replica crashes).
///     If a replica is already started and another actor sends this message, a
///     PortReadySignal will be sent free of charge in the same thread.
pub struct IcxProxy {
    logger: Logger,
    config: Config,

    // We keep the port to send to subscribers on subscription.
    // port: Option<u16>,
    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,
    //
    // /// Ready Signal subscribers.
    // ready_subscribers: Vec<Recipient<PortReadySignal>>,
}

impl IcxProxy {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        IcxProxy {
            config,
            stop_sender: None,
            thread_join: None,
            logger,
        }
    }

    fn wait_for_port_file(file_path: &Path) -> DfxResult<u16> {
        // Use a Waiter for waiting for the file to be created.
        let mut waiter = Delay::builder()
            .throttle(Duration::from_millis(100))
            .timeout(Duration::from_secs(30))
            .build();

        waiter.start();
        loop {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if let Ok(port) = content.parse::<u16>() {
                    return Ok(port);
                }
            }
            waiter
                .wait()
                .map_err(|err| anyhow!("Cannot start the replica: {:?}", err))?;
        }
    }

    fn start_icx_proxy(&mut self, addr: Addr<Self>, replica_port: u16) -> DfxResult {
        let logger = self.logger.clone();

        // Create a replica config.
        let config = &self.config.icx_proxy_config;
        let icx_proxy_pid_path = &self.config.icx_proxy_pid_path; // replica_configuration_dir.join("icx-proxy-pid");

        let icx_proxy_path = self.config.icx_proxy_path.to_path_buf();

        let (sender, receiver) = unbounded();

        let handle = icx_proxy_start_thread(
            logger,
            config.bind.clone(),
            replica_port,
            icx_proxy_path,
            icx_proxy_pid_path.clone(),
            addr,
            receiver,
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    // fn send_ready_signal(&self, port: u16) {
    //     for sub in &self.ready_subscribers {
    //         let _ = sub.do_send(PortReadySignal { port });
    //     }
    // }
}

impl Actor for IcxProxy {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let _ = self
            .config
            .port_ready_subscribe
            .do_send(PortReadySubscribe(ctx.address().recipient()));

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(self.logger, "Stopping ic-proxy...");
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(join) = self.thread_join.take() {
            let _ = join.join();
        }

        info!(self.logger, "Stopped.");
        Running::Stop
    }
}

impl Handler<PortReadySignal> for IcxProxy {
    type Result = ();

    fn handle(&mut self, msg: PortReadySignal, ctx: &mut Self::Context) {
        debug!(self.logger, "replica ready on {}", msg.port);

        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(join) = self.thread_join.take() {
            let _ = join.join();
        }

        self.start_icx_proxy(ctx.address(), msg.port)
            .expect("Could not start icx-proxy");
    }
}

// impl Handler<signals::IcxProxyRestarted> for IcxProxy {
//     type Result = ();
//
//     fn handle(&mut self, msg: IcxProxyRestarted, _ctx: &mut Self::Context) -> Self::Result {
//         self.port = Some(msg.port);
//         self.send_ready_signal(msg.port);
//     }
// }

impl Handler<Shutdown> for IcxProxy {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, _msg: Shutdown, _ctx: &mut Self::Context) -> Self::Result {
        // This is just the example for ResponseActFuture but stopping the context
        Box::pin(
            async {}
                .into_actor(self) // converts future to ActorFuture
                .map(|_, _act, ctx| {
                    ctx.stop();
                    Ok(())
                }),
        )
    }
}

enum ChildOrReceiver {
    Child,
    Receiver,
}

/// Function that waits for a child or a receiver to stop. This encapsulate the polling so
/// it is easier to maintain.
fn wait_for_child_or_receiver(
    child: &mut std::process::Child,
    receiver: &Receiver<()>,
) -> ChildOrReceiver {
    loop {
        // Check if either the child exited or a shutdown has been requested.
        // These can happen in either order in response to Ctrl-C, so increase the chance
        // to notice a shutdown request even if the replica exited quickly.
        let child_try_wait = child.try_wait();
        let receiver_signalled = receiver.recv_timeout(std::time::Duration::from_millis(100));

        match (receiver_signalled, child_try_wait) {
            (Ok(()), _) => {
                // Prefer to indicate the shutdown request
                return ChildOrReceiver::Receiver;
            }
            (Err(_), Ok(Some(_))) => {
                return ChildOrReceiver::Child;
            }
            _ => {}
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn icx_proxy_start_thread(
    logger: Logger,
    address: SocketAddr,
    replica_port: u16,
    icx_proxy_path: PathBuf,
    icx_proxy_pid_path: PathBuf,
    addr: Addr<IcxProxy>,
    receiver: Receiver<()>,
) -> DfxResult<std::thread::JoinHandle<()>> {
    let thread_handler = move || {
        // Use a Waiter for waiting for the file to be created.
        let mut waiter = Delay::builder()
            .throttle(Duration::from_millis(1000))
            .exponential_backoff(Duration::from_secs(1), 1.2)
            .build();
        waiter.start();

        // Start the process, then wait for the file.
        let icx_proxy_path = icx_proxy_path.as_os_str();

        // form the icx-proxy command here similar to replica command
        let mut cmd = std::process::Command::new(icx_proxy_path);
        let address = format!("{}", &address);
        let replica = format!("http://localhost:{}", replica_port);
        // --address 127.0.0.1:3000               (where to listen)
        // --replica http://localhost:8000/
        cmd.args(&[
            "--address",
            &address,
            "--replica",
            &replica,
        ]);
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        let mut done = false;
        while !done {
            // if let Some(port_path) = write_port_to.as_ref() {
            //     let _ = std::fs::remove_file(port_path);
            // }
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting icx-proxy...");
            let mut child = cmd.spawn().expect("Could not start icx-proxy.");

            std::fs::write(&icx_proxy_pid_path, "").expect("Could not write to icx-proxy-pid file.");
            std::fs::write(&icx_proxy_pid_path, child.id().to_string())
                .expect("Could not write to icx-proxy-pid file.");

            //addr.do_send(signals::IcxProxyRestarted { port: replica_port });

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the replica if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(logger, "Got signal to stop. Killing icx-proxy process...");
                    let _ = child.kill();
                    let _ = child.wait();
                    done = true;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "icx-proxy process failed.");
                    // Reset waiter if last start was over 2 seconds ago, and do not wait.
                    if std::time::Instant::now().duration_since(last_start)
                        >= Duration::from_secs(2)
                    {
                        debug!(
                            logger,
                            "Last icx-proxy seemed to have been healthy, not waiting..."
                        );
                        waiter.start();
                    } else {
                        // Wait before we start it again.
                        let _ = waiter.wait();
                    }
                }
            }
        }
    };

    std::thread::Builder::new()
        .name("icx-proxy-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}
