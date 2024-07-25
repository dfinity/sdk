use crate::actors::pocketic_proxy::signals::{PortReadySignal, PortReadySubscribe};
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
use slog::{debug, error, info, Logger};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;
use tempfile::tempdir;
use tokio::runtime::Builder;
use url::Url;

pub mod signals {
    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortReadySignal {
        pub url: String,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortReadySubscribe(pub Recipient<PortReadySignal>);
}

pub struct PocketIcProxyConfig {
    /// where to listen.  Becomes argument like --address 127.0.0.1:3000
    pub bind: SocketAddr,

    /// fixed replica address
    pub replica_url: Option<Url>,

    /// does the proxy need to fetch the root key
    pub fetch_root_key: bool,

    /// run pocket-ic in non-quiet mode
    pub verbose: bool,

    /// list of domains that can be served (localhost if none specified)
    pub domains: Vec<String>,
}

/// The configuration for the pocketic_proxy actor.
pub struct Config {
    pub logger: Option<Logger>,

    pub port_ready_subscribe: Option<Recipient<PortReadySubscribe>>,
    pub shutdown_controller: Addr<ShutdownController>,

    pub pocketic_proxy_config: PocketIcProxyConfig,
    pub pocketic_proxy_path: PathBuf,
    pub pocketic_proxy_pid_path: PathBuf,
    pub pocketic_proxy_port_path: PathBuf,
}

/// An actor for the PocketIC proxy webserver.  Starts/restarts pocket-ic when the replica
/// restarts (because the replica changes ports when it restarts).
pub struct PocketIcProxy {
    logger: Logger,
    config: Config,

    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,
}

impl PocketIcProxy {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        Self {
            config,
            stop_sender: None,
            thread_join: None,
            logger,
        }
    }

    fn start_pocketic_proxy(&mut self, replica_url: Url) -> DfxResult {
        let logger = self.logger.clone();
        let config = &self.config.pocketic_proxy_config;
        let pocketic_proxy_path = self.config.pocketic_proxy_path.clone();
        let pocketic_proxy_pid_path = self.config.pocketic_proxy_pid_path.clone();
        let pocketic_proxy_port_path = self.config.pocketic_proxy_port_path.clone();
        let (sender, receiver) = unbounded();

        let handle = anyhow::Context::context(
            pocketic_proxy_start_thread(
                logger,
                config.bind,
                replica_url,
                pocketic_proxy_path,
                pocketic_proxy_pid_path,
                pocketic_proxy_port_path,
                receiver,
                config.verbose,
                config.domains.clone(),
            ),
            "Failed to start PocketIC proxy thread.",
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    fn stop_pocketic_proxy(&mut self) {
        if self.stop_sender.is_some() || self.thread_join.is_some() {
            info!(self.logger, "Stopping HTTP gateway...");
            if let Some(sender) = self.stop_sender.take() {
                let _ = sender.send(());
            }

            if let Some(join) = self.thread_join.take() {
                let _ = join.join();
            }
            info!(self.logger, "Stopped.");
        }
    }

    fn wait_for_ready(port_file_path: &Path, ready_file_path: &Path) -> DfxResult<u16> {
        let mut retries = 0;
        let mut ready = false;
        loop {
            if !ready && ready_file_path.exists() {
                ready = true;
            }
            if ready {
                if let Ok(content) = std::fs::read_to_string(port_file_path) {
                    if let Ok(port) = content.parse::<u16>() {
                        return Ok(port);
                    }
                }
            }
            if retries >= 3000 {
                bail!("Cannot start pocket-ic: timed out");
            }
            std::thread::sleep(Duration::from_millis(100));
            retries += 1;
        }
    }
}

impl Actor for PocketIcProxy {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(port_ready_subscribe) = &self.config.port_ready_subscribe {
            port_ready_subscribe.do_send(PortReadySubscribe(ctx.address().recipient()));
        }

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));

        if let Some(replica_url) = &self.config.pocketic_proxy_config.replica_url {
            self.start_pocketic_proxy(replica_url.clone())
                .expect("Could not start PocketIC HTTP gateway");
        }
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.stop_pocketic_proxy();

        Running::Stop
    }
}

impl Handler<PortReadySignal> for PocketIcProxy {
    type Result = ();

    fn handle(&mut self, msg: PortReadySignal, _ctx: &mut Self::Context) {
        debug!(
            self.logger,
            "replica ready on {}, so re/starting HTTP gateway", msg.url
        );

        self.stop_pocketic_proxy();

        let replica_url = Url::parse(&msg.url).unwrap();

        self.start_pocketic_proxy(replica_url)
            .expect("Could not start PocketIC HTTP gateway");
    }
}

impl Handler<Shutdown> for PocketIcProxy {
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

fn pocketic_proxy_start_thread(
    logger: Logger,
    address: SocketAddr,
    replica_url: Url,
    pocketic_proxy_path: PathBuf,
    pocketic_proxy_pid_path: PathBuf,
    pocketic_proxy_port_path: PathBuf,
    receiver: Receiver<()>,
    verbose: bool,
    domains: Vec<String>,
) -> DfxResult<std::thread::JoinHandle<()>> {
    let thread_handler = move || {
        loop {
            // Start the process, then wait for the file.

            // form the pocket-ic command here similar to replica command
            let mut cmd = std::process::Command::new(&pocketic_proxy_path);
            if !verbose {
                cmd.env("RUST_LOG", "error");
            }

            cmd.args(["--ttl", "2592000"]);
            cmd.args(["--port-file".as_ref(), pocketic_proxy_port_path.as_os_str()]);
            let tmp = tempdir().expect("Could not create temporary directory.");
            let ready_file = tmp.path().join("ready");
            cmd.args(["--ready-file".as_ref(), ready_file.as_os_str()]);
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting pocket-ic gateway...");
            let mut child = cmd.spawn().expect("Could not start pocket-ic gateway.");

            std::fs::write(&pocketic_proxy_pid_path, "")
                .expect("Could not write to pocketic-proxy-pid file.");
            std::fs::write(&pocketic_proxy_pid_path, child.id().to_string())
                .expect("Could not write to pocketic-proxy-pid file.");
            let port =
                PocketIcProxy::wait_for_ready(&pocketic_proxy_port_path, &ready_file).unwrap();
            let _instance = match block_on_initialize_gateway(
                format!("http://localhost:{port}").parse().unwrap(),
                replica_url.clone(),
                domains.clone(),
                address,
                logger.clone(),
            ) {
                Err(e) => {
                    error!(logger, "Failed to initialize HTTP gateway: {:#}", e);
                    let _ = child.kill();
                    let _ = child.wait();
                    if receiver.try_recv().is_ok() {
                        debug!(logger, "Got signal to stop.");
                        break;
                    } else {
                        continue;
                    }
                }
                Ok(i) => i,
            };
            info!(logger, "Replica API running on {address}");

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart pocket-ic if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(
                        logger,
                        "Got signal to stop. Killing pocket-ic gateway process..."
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "pocket-ic gateway process failed.");
                    // If it took less than two seconds to exit, wait a bit before trying again.
                    if std::time::Instant::now().duration_since(last_start) < Duration::from_secs(2)
                    {
                        std::thread::sleep(Duration::from_secs(2));
                    } else {
                        debug!(
                            logger,
                            "Last pocket-ic gateway seemed to have been healthy, not waiting..."
                        );
                    }
                }
            }
        }
    };

    std::thread::Builder::new()
        .name("pocketic-proxy-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}

fn block_on_initialize_gateway(
    pocketic_url: Url,
    replica_url: Url,
    domains: Vec<String>,
    addr: SocketAddr,
    logger: Logger,
) -> DfxResult<usize> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(initialize_gateway(
            pocketic_url,
            replica_url,
            domains,
            addr,
            logger,
        ))
}

#[cfg(unix)]
async fn initialize_gateway(
    pocketic_url: Url,
    replica_url: Url,
    domains: Vec<String>,
    addr: SocketAddr,
    logger: Logger,
) -> DfxResult<usize> {
    use pocket_ic::common::rest::{
        CreateHttpGatewayResponse, HttpGatewayBackend, HttpGatewayConfig,
    };
    use reqwest::Client;
    let init_client = Client::new();
    let resp = init_client
        .post(pocketic_url.join("http_gateway").unwrap())
        .json(&HttpGatewayConfig {
            forward_to: HttpGatewayBackend::Replica(replica_url.to_string()),
            //TODO use the rest of the addr
            listen_at: Some(addr.port()),
            domains: Some(domains),
            https_config: None,
        })
        .send()
        .await?
        .error_for_status()?;
    let resp = resp.json::<CreateHttpGatewayResponse>().await?;
    let instance = match resp {
        CreateHttpGatewayResponse::Error { message } => bail!("Gateway init error: {message}"),
        CreateHttpGatewayResponse::Created(info) => info.instance_id,
    };
    info!(logger, "Initialized HTTP gateway.");
    Ok(instance)
}

#[cfg(not(unix))]
async fn initialize_gateway(
    _: Url,
    _: Url,
    _: Vec<Url>,
    _: SocketAddr,
    _: Logger,
) -> DfxResult<usize> {
    bail!("PocketIC gateway not supported on this platform")
}
