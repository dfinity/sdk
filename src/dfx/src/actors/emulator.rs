use crate::actors::icx_proxy::signals::{PortReadySignal, PortReadySubscribe};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::{DfxError, DfxResult};

use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Recipient,
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

    /// A message sent to the Emulator when the process is restarted. Since we're
    /// restarting inside our own actor, this message should not be exposed.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct EmulatorRestarted {
        pub port: u16,
    }
}

/// The configuration for the emulator actor.
#[derive(Clone)]
pub struct Config {
    pub ic_ref_path: PathBuf,
    pub write_port_to: PathBuf,
    pub shutdown_controller: Addr<ShutdownController>,
    pub logger: Option<Logger>,
}

/// A emulator actor. Starts the emulator, can subscribe to a Ready signal and a
/// Killed signal.
/// This starts a thread that monitors the process and send signals to any subscriber
/// listening for restarts. The message contains the port the emulator is listening to.
///
/// Signals
///   - PortReadySubscribe
///     Subscribe a recipient (address) to receive a EmulatorReadySignal message when
///     the emulator is ready to listen to a port. The message can be sent multiple
///     times (e.g. if the emulator crashes).
///     If a emulator is already started and another actor sends this message, a
///     EmulatorReadySignal will be sent free of charge in the same thread.
pub struct Emulator {
    logger: Logger,
    config: Config,

    // We keep the port to send to subscribers on subscription.
    port: Option<u16>,
    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,

    /// Ready Signal subscribers.
    ready_subscribers: Vec<Recipient<PortReadySignal>>,
}

impl Emulator {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        Emulator {
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
                .map_err(|err| anyhow!("Cannot start ic-ref: {:?}", err))?;
        }
    }

    fn start_emulator(&mut self, addr: Addr<Self>) -> DfxResult {
        let logger = self.logger.clone();

        let (sender, receiver) = unbounded();

        let handle = anyhow::Context::context(
            emulator_start_thread(logger, self.config.clone(), addr, receiver),
            "Failed to start emulator thread.",
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

impl Actor for Emulator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_emulator(ctx.address())
            .expect("Could not start the emulator");

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(self.logger, "Stopping ic-ref...");
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

impl Handler<PortReadySubscribe> for Emulator {
    type Result = ();

    fn handle(&mut self, msg: PortReadySubscribe, _: &mut Self::Context) {
        // If we have a port, send that we're already ready! Yeah!
        if let Some(port) = self.port {
            let _ = msg.0.do_send(PortReadySignal { port });
        }

        self.ready_subscribers.push(msg.0);
    }
}

impl Handler<signals::EmulatorRestarted> for Emulator {
    type Result = ();

    fn handle(
        &mut self,
        msg: signals::EmulatorRestarted,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.port = Some(msg.port);
        self.send_ready_signal(msg.port);
    }
}

impl Handler<Shutdown> for Emulator {
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

fn emulator_start_thread(
    logger: Logger,
    config: Config,
    addr: Addr<Emulator>,
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
        let ic_ref_path = config.ic_ref_path.as_os_str();

        // form the ic-start command here similar to emulator command
        let mut cmd = std::process::Command::new(ic_ref_path);
        cmd.args(&["--pick-port"]);
        cmd.args(&[
            "--write-port-to",
            &config.write_port_to.to_string_lossy().to_string(),
        ]);
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        let mut done = false;
        while !done {
            let _ = std::fs::remove_file(&config.write_port_to);
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting emulator...");
            let mut child = cmd.spawn().expect("Could not start emulator.");

            let port = Emulator::wait_for_port_file(&config.write_port_to).unwrap();
            addr.do_send(signals::EmulatorRestarted { port });

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the emulator if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(logger, "Got signal to stop. Killing emulator process...");
                    let _ = child.kill();
                    let _ = child.wait();
                    done = true;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "Emulator process failed.");
                    // Reset waiter if last start was over 2 seconds ago, and do not wait.
                    if std::time::Instant::now().duration_since(last_start)
                        >= Duration::from_secs(2)
                    {
                        debug!(
                            logger,
                            "Last emulator seemed to have been healthy, not waiting..."
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
        .name("emulator-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}
