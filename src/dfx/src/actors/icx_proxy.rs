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
use crossbeam::channel::{unbounded, Receiver, Sender};
use garcon::{Delay, Waiter};
use slog::{debug, info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::Duration;
use url::Url;

pub mod signals {
    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortReadySignal {
        pub port: u16,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortReadySubscribe(pub Recipient<PortReadySignal>);
}

pub struct IcxProxyConfig {
    /// where to listen.  Becomes argument like --address 127.0.0.1:3000
    pub bind: SocketAddr,

    /// fixed replica addresses
    pub replica_urls: Vec<Url>,

    /// does the icx-proxy need to fetch the root key
    pub fetch_root_key: bool,
}

/// The configuration for the icx_proxy actor.
pub struct Config {
    pub logger: Option<Logger>,

    pub port_ready_subscribe: Option<Recipient<PortReadySubscribe>>,
    pub shutdown_controller: Addr<ShutdownController>,

    pub icx_proxy_config: IcxProxyConfig,
    pub icx_proxy_path: PathBuf,
    pub icx_proxy_pid_path: PathBuf,
}

/// An actor for the icx-proxy webserver.  Starts/restarts icx-proxy when the replica
/// restarts (because the replica changes ports when it restarts).
pub struct IcxProxy {
    logger: Logger,
    config: Config,

    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,
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

    fn start_icx_proxy(&mut self, replica_urls: Vec<Url>) -> DfxResult {
        let logger = self.logger.clone();
        let config = &self.config.icx_proxy_config;
        let icx_proxy_pid_path = &self.config.icx_proxy_pid_path;
        let icx_proxy_path = self.config.icx_proxy_path.to_path_buf();
        let fetch_root_key = config.fetch_root_key;
        let (sender, receiver) = unbounded();

        let handle = anyhow::Context::context(
            icx_proxy_start_thread(
                logger,
                config.bind,
                replica_urls,
                icx_proxy_path,
                icx_proxy_pid_path.clone(),
                receiver,
                fetch_root_key,
            ),
            "Failed to start ICX proxy thread.",
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    fn stop_icx_proxy(&mut self) {
        if self.stop_sender.is_some() || self.thread_join.is_some() {
            info!(self.logger, "Stopping icx-proxy...");
            if let Some(sender) = self.stop_sender.take() {
                let _ = sender.send(());
            }

            if let Some(join) = self.thread_join.take() {
                let _ = join.join();
            }
            info!(self.logger, "Stopped.");
        }
    }
}

impl Actor for IcxProxy {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(port_ready_subscribe) = &self.config.port_ready_subscribe {
            let _ = port_ready_subscribe.do_send(PortReadySubscribe(ctx.address().recipient()));
        }

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));

        if !self.config.icx_proxy_config.replica_urls.is_empty() {
            self.start_icx_proxy(self.config.icx_proxy_config.replica_urls.clone())
                .expect("Could not start icx-proxy");
        }
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.stop_icx_proxy();

        Running::Stop
    }
}

impl Handler<PortReadySignal> for IcxProxy {
    type Result = ();

    fn handle(&mut self, msg: PortReadySignal, _ctx: &mut Self::Context) {
        debug!(
            self.logger,
            "replica ready on {}, so re/starting icx-proxy", msg.port
        );

        self.stop_icx_proxy();

        let replica_url = format!("http://localhost:{}", msg.port);
        let replica_urls = vec![Url::parse(&replica_url).unwrap()];

        self.start_icx_proxy(replica_urls)
            .expect("Could not start icx-proxy");
    }
}

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

#[allow(clippy::too_many_arguments)]
fn icx_proxy_start_thread(
    logger: Logger,
    address: SocketAddr,
    replica_urls: Vec<Url>,
    icx_proxy_path: PathBuf,
    icx_proxy_pid_path: PathBuf,
    receiver: Receiver<()>,
    fetch_root_key: bool,
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
        if fetch_root_key {
            cmd.arg("--fetch-root-key");
        }
        let address = format!("{}", &address);
        cmd.args(&["--address", &address]);
        for url in replica_urls {
            let s = format!("{}", url);
            cmd.args(&["--replica", &s]);
        }
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        let mut done = false;
        while !done {
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting icx-proxy...");
            let mut child = cmd.spawn().expect("Could not start icx-proxy.");

            std::fs::write(&icx_proxy_pid_path, "")
                .expect("Could not write to icx-proxy-pid file.");
            std::fs::write(&icx_proxy_pid_path, child.id().to_string())
                .expect("Could not write to icx-proxy-pid file.");

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the icx-proxy if done = true.
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
