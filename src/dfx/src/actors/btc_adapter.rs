use crate::actors::btc_adapter::signals::{BtcAdapterReady, BtcAdapterReadySubscribe};
use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::{DfxError, DfxResult};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Recipient,
    ResponseActFuture, Running, WrapFuture,
};
use anyhow::bail;
use crossbeam::channel::{unbounded, Receiver, Sender};
use slog::{debug, info, Logger};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;

pub mod signals {
    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct BtcAdapterReady {}

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct BtcAdapterReadySubscribe(pub Recipient<BtcAdapterReady>);
}

#[derive(Clone)]
pub struct Config {
    pub btc_adapter_path: PathBuf,

    pub config_path: PathBuf,
    pub socket_path: Option<PathBuf>,
    pub shutdown_controller: Addr<ShutdownController>,
    pub btc_adapter_pid_file_path: PathBuf,

    pub logger: Option<Logger>,
}

/// An actor for the ic-btc-adapter process.  Publishes information about
/// the process starting or restarting, so that other processes can reconnect.
pub struct BtcAdapter {
    config: Config,

    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,

    ready: bool,
    ready_subscribers: Vec<Recipient<BtcAdapterReady>>,

    logger: Logger,
}

impl BtcAdapter {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        BtcAdapter {
            config,
            stop_sender: None,
            thread_join: None,
            ready: false,
            ready_subscribers: Vec::new(),
            logger,
        }
    }

    /// Wait for BTC adapter creating the socket file.
    /// Retry every 0.1s for 5 minutes.
    /// Will break out of the loop if receive stop signal.
    ///
    /// Returns
    /// - Ok(Some()) if succeed;
    /// - Ok(None) if receive stop signal (`dfx start` then Ctrl-C immediately);
    /// - Err if time out;
    fn wait_for_socket(socket_path: &Path, stop_receiver: &Receiver<()>) -> DfxResult<Option<()>> {
        let mut retries = 0;
        while !socket_path.exists() {
            if stop_receiver.try_recv().is_ok() {
                return Ok(None);
            }
            if retries >= 3000 {
                bail!("Cannot start btc-adapter: timed out");
            }
            std::thread::sleep(Duration::from_millis(100));
            retries += 1;
        }
        Ok(Some(()))
    }

    fn start_btc_adapter(&mut self, addr: Addr<Self>) -> DfxResult {
        let logger = self.logger.clone();

        let (sender, receiver) = unbounded();

        let handle = anyhow::Context::context(
            btc_adapter_start_thread(logger, self.config.clone(), addr, receiver),
            "Failed to start BTC adapter thread.",
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    fn send_ready_signal(&self) {
        for sub in &self.ready_subscribers {
            sub.do_send(BtcAdapterReady {});
        }
    }
}

impl Actor for BtcAdapter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_btc_adapter(ctx.address())
            .expect("Could not start btc-adapter");

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(self.logger, "Stopping btc-adapter...");
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

impl Handler<signals::BtcAdapterReady> for BtcAdapter {
    type Result = ();

    fn handle(&mut self, _msg: signals::BtcAdapterReady, _ctx: &mut Self::Context) -> Self::Result {
        self.ready = true;
        self.send_ready_signal();
    }
}

impl Handler<BtcAdapterReadySubscribe> for BtcAdapter {
    type Result = ();

    fn handle(&mut self, msg: BtcAdapterReadySubscribe, _: &mut Self::Context) {
        // If the adapter is already ready, let the new subscriber know! Yeah!
        if self.ready {
            msg.0.do_send(BtcAdapterReady {});
        }

        self.ready_subscribers.push(msg.0);
    }
}

impl Handler<Shutdown> for BtcAdapter {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, _msg: Shutdown, _ctx: &mut Self::Context) -> Self::Result {
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

fn btc_adapter_start_thread(
    logger: Logger,
    config: Config,
    addr: Addr<BtcAdapter>,
    receiver: Receiver<()>,
) -> DfxResult<std::thread::JoinHandle<()>> {
    let thread_handler = move || {
        let btc_adapter_path = config.btc_adapter_path.as_os_str();
        let mut cmd = std::process::Command::new(btc_adapter_path);
        cmd.arg(&config.config_path.to_string_lossy().to_string());

        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        loop {
            if let Some(socket_path) = &config.socket_path {
                if socket_path.exists() {
                    std::fs::remove_file(socket_path).expect("Could not remove btc-adapter socket");
                }
            }
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting ic-btc-adapter...");
            let mut child = cmd.spawn().expect("Could not start ic-btc-adapter.");

            std::fs::write(&config.btc_adapter_pid_file_path, "")
                .expect("Could not write to btc-adapter-pid file.");
            std::fs::write(&config.btc_adapter_pid_file_path, child.id().to_string())
                .expect("Could not write to btc-adapter-pid file.");

            if let Some(socket_path) = &config.socket_path {
                // If Ctrl-C right after `dfx start`, the adapter child process will be killed already.
                // And the socket file will never be ready.
                // So we let `wait_for_socket` method to break out from the waiting,
                // finish this actor starting ASAP and let the system stop the actor.
                if BtcAdapter::wait_for_socket(socket_path, &receiver)
                    .expect("btc adapter socket was not created")
                    .is_none()
                {
                    break;
                }
            }
            addr.do_send(signals::BtcAdapterReady {});

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the adapter if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver(()) => {
                    debug!(logger, "Got signal to stop. Killing btc-adapter process...");
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "ic-btc-adapter process failed.");
                    // If it took less than two seconds to exit, wait a bit before trying again.
                    if std::time::Instant::now().duration_since(last_start) < Duration::from_secs(2)
                    {
                        std::thread::sleep(Duration::from_secs(2));
                    } else {
                        debug!(
                            logger,
                            "Last ic-btc-adapter seemed to have been healthy, not waiting..."
                        );
                    }
                }
            }
        }
    };

    std::thread::Builder::new()
        .name("btc-adapter-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}
