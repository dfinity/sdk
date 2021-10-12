use crate::actors::icx_proxy::signals::{PortReadySignal, PortReadySubscribe};
use crate::actors::replica::signals::ReplicaRestarted;
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::replica_config::ReplicaConfig;

use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, Handler, Recipient,
    ResponseActFuture, Running, WrapFuture,
};
use anyhow::anyhow;
use crossbeam::channel::{unbounded, Receiver, Sender};
use garcon::{Delay, Waiter};
use slog::{debug, info, Logger};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;

pub mod signals {
    use actix::prelude::*;

    /// A message sent to the Replica when the process is restarted. Since we're
    /// restarting inside our own actor, this message should not be exposed.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct ReplicaRestarted {
        pub port: u16,
    }
}

/// The configuration for the replica actor.
pub struct Config {
    pub ic_starter_path: PathBuf,
    pub replica_config: ReplicaConfig,
    pub replica_path: PathBuf,
    pub shutdown_controller: Addr<ShutdownController>,
    pub logger: Option<Logger>,
    pub replica_configuration_dir: PathBuf,
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
pub struct Replica {
    logger: Logger,
    config: Config,

    // We keep the port to send to subscribers on subscription.
    port: Option<u16>,
    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,

    /// Ready Signal subscribers.
    ready_subscribers: Vec<Recipient<PortReadySignal>>,
}

impl Replica {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        Replica {
            config,
            port: None,
            stop_sender: None,
            thread_join: None,
            ready_subscribers: Vec::new(),
            logger,
        }
    }

    fn wait_for_port_file(file_path: &Path) -> DfxResult<u16> {
        // Use a Waiter for waiting for the file to be created.
        let mut waiter = Delay::builder()
            .throttle(Duration::from_millis(100))
            .timeout(Duration::from_secs(120))
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

    fn start_replica(&mut self, addr: Addr<Self>) -> DfxResult {
        let logger = self.logger.clone();

        // Create a replica config.
        let config = &self.config.replica_config;
        let replica_pid_path = self.config.replica_configuration_dir.join("replica-pid");

        let port = config.http_handler.port;
        let write_port_to = config.http_handler.write_port_to.clone();
        let replica_path = self.config.replica_path.to_path_buf();
        let ic_starter_path = self.config.ic_starter_path.to_path_buf();

        let (sender, receiver) = unbounded();

        let handle = replica_start_thread(
            logger,
            config.clone(),
            port,
            write_port_to,
            ic_starter_path,
            replica_path,
            replica_pid_path,
            addr,
            receiver,
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    fn send_ready_signal(&self, port: u16) {
        for sub in &self.ready_subscribers {
            let _ = sub.do_send(PortReadySignal { port });
        }
    }
}

impl Actor for Replica {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_replica(ctx.address())
            .expect("Could not start the replica");

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(self.logger, "Stopping the replica...");
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

impl Handler<PortReadySubscribe> for Replica {
    type Result = ();

    fn handle(&mut self, msg: PortReadySubscribe, _: &mut Self::Context) {
        // If we have a port, send that we're already ready! Yeah!
        if let Some(port) = self.port {
            let _ = msg.0.do_send(PortReadySignal { port });
        }

        self.ready_subscribers.push(msg.0);
    }
}

impl Handler<signals::ReplicaRestarted> for Replica {
    type Result = ();

    fn handle(&mut self, msg: ReplicaRestarted, _ctx: &mut Self::Context) -> Self::Result {
        self.port = Some(msg.port);
        self.send_ready_signal(msg.port);
    }
}

impl Handler<Shutdown> for Replica {
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

#[allow(clippy::too_many_arguments)]
fn replica_start_thread(
    logger: Logger,
    config: ReplicaConfig,
    port: Option<u16>,
    write_port_to: Option<PathBuf>,
    ic_starter_path: PathBuf,
    replica_path: PathBuf,
    replica_pid_path: PathBuf,
    addr: Addr<Replica>,
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
        let ic_starter_path = ic_starter_path.as_os_str();

        // form the ic-start command here similar to replica command
        let mut cmd = std::process::Command::new(ic_starter_path);
        cmd.args(&[
            "--replica-path",
            replica_path.to_str().unwrap_or_default(),
            "--state-dir",
            config.state_manager.state_root.to_str().unwrap_or_default(),
            "--create-funds-whitelist",
            "*",
            "--consensus-pool-backend",
            "rocksdb",
        ]);
        if let Some(port) = port {
            cmd.args(&["--http-port", &port.to_string()]);
        }
        if let Some(write_port_to) = &write_port_to {
            cmd.args(&[
                "--http-port-file",
                &write_port_to.to_string_lossy().to_string(),
            ]);
        }
        cmd.args(&[
            "--initial-notary-delay-millis",
            // The intial notary delay is set to 2500ms in the replica's
            // default subnet configuration to help running tests.
            // For our production network, we actually set them to 600ms.
            "600",
        ]);
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        let mut done = false;
        while !done {
            if let Some(port_path) = write_port_to.as_ref() {
                let _ = std::fs::remove_file(port_path);
            }
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting replica...");
            let mut child = cmd.spawn().expect("Could not start replica.");

            std::fs::write(&replica_pid_path, "").expect("Could not write to replica-pid file.");
            std::fs::write(&replica_pid_path, child.id().to_string())
                .expect("Could not write to replica-pid file.");

            let port = port.unwrap_or_else(|| {
                Replica::wait_for_port_file(write_port_to.as_ref().unwrap()).unwrap()
            });
            addr.do_send(signals::ReplicaRestarted { port });

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the replica if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(logger, "Got signal to stop. Killing replica process...");
                    let _ = child.kill();
                    let _ = child.wait();
                    done = true;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "Replica process failed.");
                    // Reset waiter if last start was over 2 seconds ago, and do not wait.
                    if std::time::Instant::now().duration_since(last_start)
                        >= Duration::from_secs(2)
                    {
                        debug!(
                            logger,
                            "Last replica seemed to have been healthy, not waiting..."
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
        .name("replica-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}
