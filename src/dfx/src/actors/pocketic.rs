#![cfg_attr(windows, allow(unused))]

use crate::actors::post_start::signals::{PocketIcReadySignal, PocketIcReadySubscribe};
use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::{DfxError, DfxResult};
#[cfg(unix)]
use crate::lib::info::replica_rev;
#[cfg(unix)]
use crate::lib::integrations::bitcoin::initialize_bitcoin_canister;
#[cfg(unix)]
use crate::lib::integrations::create_integrations_agent;
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Recipient,
    ResponseActFuture, Running, WrapFuture,
};
use anyhow::{anyhow, bail};
#[cfg(unix)]
use candid::Principal;
use crossbeam::channel::{unbounded, Receiver, Sender};
#[cfg(unix)]
use dfx_core::config::model::replica_config::CachedConfig;
use dfx_core::config::model::replica_config::ReplicaConfig;
#[cfg(unix)]
use dfx_core::json::save_json_file;
use slog::{debug, error, warn, Logger};
use std::net::SocketAddr;
use std::ops::ControlFlow::{self, *};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use url::Url;

pub mod signals {
    use actix::prelude::*;

    /// A message sent to the PocketIc when the process is restarted. Since we're
    /// restarting inside our own actor, this message should not be exposed.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct PocketIcRestarted;
}

#[derive(Clone)]
pub struct PocketIcProxyConfig {
    /// where to listen.  Becomes argument like --address 127.0.0.1:3000
    pub bind: SocketAddr,

    /// list of domains that can be served (localhost if none specified)
    pub domains: Option<Vec<String>>,
}

/// The configuration for the PocketIC actor.
#[derive(Clone)]
pub struct Config {
    pub pocketic_path: PathBuf,
    pub effective_config_path: PathBuf,
    pub replica_config: ReplicaConfig,
    pub bitcoind_addr: Option<Vec<SocketAddr>>,
    pub bitcoin_integration_config: Option<BitcoinIntegrationConfig>,
    pub port: Option<u16>,
    pub port_file: PathBuf,
    pub pid_file: PathBuf,
    pub shutdown_controller: Addr<ShutdownController>,
    pub logger: Option<Logger>,
    pub pocketic_proxy_config: PocketIcProxyConfig,
    pub docker: Option<String>,
}

#[derive(Clone)]
pub struct BitcoinIntegrationConfig {
    pub canister_init_arg: String,
}

/// A PocketIC actor. Starts the server, can subscribe to a Ready signal and a
/// Killed signal.
/// This starts a thread that monitors the process and send signals to any subscriber
/// listening for restarts. The message contains the port the server is listening to.
///
/// Signals
///   - PocketIcReadySubscribe
///     Subscribe a recipient (address) to receive a PocketIcReadySignal message when
///     the server is ready to listen to a port. The message can be sent multiple
///     times (e.g. if the server crashes).
///     If a server is already started and another actor sends this message, a
///     PocketIcReadySignal will be sent free of charge in the same thread.
pub struct PocketIc {
    logger: Logger,
    config: Config,

    // We keep the port to send to subscribers on subscription.
    port: Option<u16>,
    stop_sender: Option<Sender<()>>,
    thread_join: Option<JoinHandle<()>>,

    /// Ready Signal subscribers.
    ready_subscribers: Vec<Recipient<PocketIcReadySignal>>,
}

impl PocketIc {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        Self {
            config,
            port: None,
            stop_sender: None,
            thread_join: None,
            ready_subscribers: Vec::new(),
            logger,
        }
    }

    fn wait_for_ready(
        port_file_path: &Path,
        shutdown_signal: Receiver<()>,
        docker_container_id: Option<String>,
    ) -> Result<u16, ControlFlow<(), DfxError>> {
        let mut retries = 0;
        loop {
            if let Some(container_id) = docker_container_id.as_ref() {
                let output = std::process::Command::new("docker")
                    .args(["exec", container_id.as_str(), "cat", "pocket-ic-port"])
                    .output();
                if let Ok(output) = output {
                    if let Ok(port) = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .parse::<u16>()
                    {
                        return Ok(port);
                    }
                }
            } else if let Ok(content) = std::fs::read_to_string(port_file_path) {
                if content.ends_with('\n') {
                    if let Ok(port) = content.trim().parse::<u16>() {
                        return Ok(port);
                    }
                }
            }

            if shutdown_signal.try_recv().is_ok() {
                return Err(Break(()));
            }
            if retries >= 3000 {
                return Err(Continue(anyhow!("Timed out")));
            }
            std::thread::sleep(Duration::from_millis(100));
            retries += 1;
        }
    }

    fn start_pocketic(&mut self, addr: Addr<Self>) -> DfxResult {
        let logger = self.logger.clone();

        let (sender, receiver) = unbounded();

        let handle = anyhow::Context::context(
            pocketic_start_thread(logger, self.config.clone(), addr, receiver),
            "Failed to start PocketIC thread.",
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    fn send_ready_signal(&self) {
        for sub in &self.ready_subscribers {
            sub.do_send(PocketIcReadySignal {
                address: self.config.pocketic_proxy_config.bind,
            });
        }
    }
}

impl Actor for PocketIc {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_pocketic(ctx.address())
            .expect("Could not start PocketIC");

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        debug!(self.logger, "Stopping PocketIC...");
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(join) = self.thread_join.take() {
            let _ = join.join();
        }

        debug!(self.logger, "Stopped.");
        Running::Stop
    }
}

impl Handler<PocketIcReadySubscribe> for PocketIc {
    type Result = ();

    fn handle(&mut self, msg: PocketIcReadySubscribe, _: &mut Self::Context) {
        // If we have a port, send that we're already ready! Yeah!
        if self.port.is_some() {
            msg.0.do_send(PocketIcReadySignal {
                address: self.config.pocketic_proxy_config.bind,
            });
        }

        self.ready_subscribers.push(msg.0);
    }
}

impl Handler<signals::PocketIcRestarted> for PocketIc {
    type Result = ();

    fn handle(
        &mut self,
        _msg: signals::PocketIcRestarted,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_ready_signal();
    }
}

impl Handler<Shutdown> for PocketIc {
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

fn pocketic_start_thread(
    logger: Logger,
    config: Config,
    addr: Addr<PocketIc>,
    receiver: Receiver<()>,
) -> DfxResult<std::thread::JoinHandle<()>> {
    let thread_handler = move || {
        loop {
            let mut bind_address = config.pocketic_proxy_config.bind;
            let (mut child, docker_container_id, last_start) =
                if let Some(docker_image) = config.docker.as_ref() {
                    // Run the container.
                    // TODO: Check if docker command is available.
                    let mut cmd = std::process::Command::new("docker");
                    cmd.args(["run"]);
                    bind_address = convert_to_docker_bind_address(bind_address);
                    cmd.args([
                        "-p",
                        format!("{}:{}", bind_address.port(), bind_address.port()).as_str(),
                        "-p",
                        "8081:8081",
                    ]);
                    cmd.args(["-d", docker_image.as_str()]);
                    cmd.stdout(std::process::Stdio::piped());

                    let last_start = std::time::Instant::now();
                    let child = cmd.spawn().expect("Could not start PocketIC.");

                    // Retrieve the container id.
                    // TODO: The container id will be used to remove the container when the process stops.
                    let output = child.wait_with_output().unwrap();
                    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

                    // Stream the logs.
                    let mut cmd = std::process::Command::new("docker");
                    cmd.args(["logs", "-f", container_id.as_ref()]);
                    cmd.stdout(std::process::Stdio::inherit());
                    let child = cmd.spawn().expect("Could not stream logs.");

                    (child, Some(container_id), last_start)
                } else {
                    // Start the process, then wait for the file.
                    let pocketic_path = config.pocketic_path.as_os_str();

                    // form the pocket-ic command here similar to the ic-starter command
                    let mut cmd = std::process::Command::new(pocketic_path);
                    if let Some(port) = config.port {
                        cmd.args(["--port", &port.to_string()]);
                    };
                    cmd.args([
                        "--port-file",
                        &config.port_file.to_string_lossy(),
                        "--ttl",
                        "2592000",
                    ]);
                    cmd.args(["--log-levels", "error"]);
                    cmd.stdout(std::process::Stdio::inherit());
                    cmd.stderr(std::process::Stdio::inherit());
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::CommandExt;
                        cmd.process_group(0);
                    }
                    let _ = std::fs::remove_file(&config.port_file);
                    let last_start = std::time::Instant::now();
                    debug!(logger, "Starting PocketIC...");
                    let child = cmd.spawn().expect("Could not start PocketIC.");

                    if let Err(e) = std::fs::write(&config.pid_file, child.id().to_string()) {
                        warn!(
                            logger,
                            "Failed to write PocketIC PID to {}: {e}",
                            config.pid_file.display()
                        );
                    }

                    (child, None, last_start)
                };

            let port = match PocketIc::wait_for_ready(
                &config.port_file,
                receiver.clone(),
                docker_container_id,
            ) {
                Ok(p) => p,
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    if let Continue(e) = e {
                        error!(logger, "Failed to start pocket-ic: {e:#}");
                        continue;
                    } else {
                        debug!(logger, "Got signal to stop");
                        break;
                    }
                }
            };
            println!("port: {}", port);

            let server_instance = match initialize_pocketic(
                port,
                &config.effective_config_path,
                &config.bitcoind_addr,
                &config.bitcoin_integration_config,
                &config.replica_config,
                logger.clone(),
            ) {
                Err(e) => {
                    error!(logger, "Failed to initialize PocketIC: {e:#}");

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

            let gateway_instance = match initialize_gateway(
                format!("http://localhost:{port}").parse().unwrap(),
                server_instance,
                config.pocketic_proxy_config.domains.clone(),
                bind_address,
                logger.clone(),
            ) {
                Err(e) => {
                    error!(logger, "Failed to initialize HTTP gateway: {e:#}");
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

            addr.do_send(signals::PocketIcRestarted);
            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the server if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(logger, "Got signal to stop. Killing PocketIC process...");
                    if let Err(e) =
                        shutdown_pocketic(port, server_instance, gateway_instance, logger.clone())
                    {
                        error!(logger, "Error shutting down PocketIC gracefully: {e}");
                    }
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "PocketIC process failed.");
                    // If it took less than two seconds to exit, wait a bit before trying again.
                    if Instant::now().duration_since(last_start) < Duration::from_secs(2) {
                        std::thread::sleep(Duration::from_secs(2));
                    } else {
                        debug!(
                            logger,
                            "Last PocketIC seemed to have been healthy, not waiting..."
                        );
                    }
                }
            }
        }
    };

    std::thread::Builder::new()
        .name("pocketic-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}

fn convert_to_docker_bind_address(address: SocketAddr) -> SocketAddr {
    let mut bind_address = address;
    if bind_address.is_ipv6() {
        bind_address.set_ip(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED));
    } else {
        bind_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
    }
    bind_address
}

#[cfg(unix)]
#[tokio::main(flavor = "current_thread")]
async fn initialize_pocketic(
    port: u16,
    effective_config_path: &Path,
    bitcoind_addr: &Option<Vec<SocketAddr>>,
    bitcoin_integration_config: &Option<BitcoinIntegrationConfig>,
    replica_config: &ReplicaConfig,
    logger: Logger,
) -> DfxResult<usize> {
    use dfx_core::config::model::dfinity::ReplicaSubnetType;
    use pocket_ic::common::rest::{
        AutoProgressConfig, CreateInstanceResponse, ExtendedSubnetConfigSet, InstanceConfig,
        RawTime, SubnetSpec,
    };
    use reqwest::Client;
    use time::OffsetDateTime;
    let init_client = Client::new();
    debug!(logger, "Configuring PocketIC server");
    let mut subnet_config_set = ExtendedSubnetConfigSet {
        nns: Some(SubnetSpec::default()),
        sns: Some(SubnetSpec::default()),
        ii: Some(SubnetSpec::default()),
        fiduciary: Some(SubnetSpec::default()),
        bitcoin: Some(SubnetSpec::default()),
        system: vec![],
        verified_application: vec![],
        application: vec![],
    };
    match replica_config.subnet_type {
        ReplicaSubnetType::Application => subnet_config_set.application.push(<_>::default()),
        ReplicaSubnetType::System => subnet_config_set.system.push(<_>::default()),
        ReplicaSubnetType::VerifiedApplication => {
            subnet_config_set.verified_application.push(<_>::default())
        }
    }
    let resp = init_client
        .post(format!("http://localhost:{port}/instances"))
        .json(&InstanceConfig {
            subnet_config_set,
            state_dir: Some(replica_config.state_manager.state_root.clone()),
            nonmainnet_features: true,
            log_level: Some(replica_config.log_level.to_pocketic_string()),
            bitcoind_addr: bitcoind_addr.clone(),
        })
        .send()
        .await?
        .error_for_status()?
        .json::<CreateInstanceResponse>()
        .await?;
    let instance = match resp {
        CreateInstanceResponse::Error { message } => {
            bail!("PocketIC init error: {message}");
        }
        CreateInstanceResponse::Created {
            instance_id,
            topology,
        } => {
            let default_effective_canister_id: Principal =
                topology.default_effective_canister_id.into();
            let effective_config = CachedConfig::pocketic(
                replica_config,
                replica_rev().into(),
                Some(default_effective_canister_id),
            );
            save_json_file(effective_config_path, &effective_config)?;
            instance_id
        }
    };
    init_client
        .post(format!(
            "http://localhost:{port}/instances/{instance}/update/set_time"
        ))
        .json(&RawTime {
            nanos_since_epoch: OffsetDateTime::now_utc()
                .unix_timestamp_nanos()
                .try_into()
                .unwrap(),
        })
        .send()
        .await?
        .error_for_status()?;
    init_client
        .post(format!(
            "http://localhost:{port}/instances/{instance}/auto_progress"
        ))
        .json(&AutoProgressConfig {
            artificial_delay_ms: Some(replica_config.artificial_delay as u64),
        })
        .send()
        .await?
        .error_for_status()?;

    let agent_url = format!("http://localhost:{port}/instances/{instance}/");

    debug!(logger, "Waiting for replica to report healthy status");
    crate::lib::replica::status::ping_and_wait(&agent_url).await?;

    if let Some(bitcoin_integration_config) = bitcoin_integration_config {
        let agent = create_integrations_agent(&agent_url, &logger).await?;
        initialize_bitcoin_canister(&agent, &logger, bitcoin_integration_config.clone()).await?;
    }

    debug!(logger, "Initialized PocketIC.");
    Ok(instance)
}

#[cfg(not(unix))]
fn initialize_pocketic(
    _: u16,
    _: &Path,
    _: &Option<Vec<SocketAddr>>,
    _: &Option<BitcoinIntegrationConfig>,
    _: &ReplicaConfig,
    _: Logger,
) -> DfxResult<usize> {
    bail!("PocketIC not supported on this platform")
}

#[cfg(unix)]
#[tokio::main(flavor = "current_thread")]
async fn initialize_gateway(
    server_url: Url,
    server_instance: usize,
    domains: Option<Vec<String>>,
    addr: SocketAddr,
    logger: Logger,
) -> DfxResult<usize> {
    use pocket_ic::common::rest::{
        CreateHttpGatewayResponse, HttpGatewayBackend, HttpGatewayConfig,
    };
    use reqwest::Client;
    let init_client = Client::new();
    debug!(logger, "Configuring PocketIC gateway");
    let resp = init_client
        .post(server_url.join("http_gateway").unwrap())
        .json(&HttpGatewayConfig {
            forward_to: HttpGatewayBackend::Replica(
                server_url
                    .join(&format!("instances/{server_instance}/"))
                    .unwrap()
                    .to_string(),
            ),
            ip_addr: Some(addr.ip().to_string()),
            port: Some(addr.port()),
            domains,
            https_config: None,
        })
        .send()
        .await?
        .error_for_status()?;
    let resp = resp.json::<CreateHttpGatewayResponse>().await?;
    let instance = match resp {
        CreateHttpGatewayResponse::Created(info) => info.instance_id,
        CreateHttpGatewayResponse::Error { message } => bail!("Gateway init error: {message}"),
    };
    debug!(logger, "Initialized HTTP gateway.");
    Ok(instance)
}

#[cfg(not(unix))]
fn initialize_gateway(
    _: Url,
    _: usize,
    _: Option<Vec<String>>,
    _: SocketAddr,
    _: Logger,
) -> DfxResult<usize> {
    bail!("PocketIC gateway not supported on this platform")
}

#[cfg(unix)]
#[tokio::main(flavor = "current_thread")]
async fn shutdown_pocketic(
    port: u16,
    server_instance: usize,
    gateway_instance: usize,
    logger: Logger,
) -> DfxResult {
    use reqwest::Client;
    let shutdown_client = Client::new();
    debug!(logger, "Sending shutdown request to PocketIC server");
    shutdown_client
        .post(format!(
            "http://localhost:{port}/http_gateway/{gateway_instance}/stop"
        ))
        .send()
        .await?
        .error_for_status()?;
    shutdown_client
        .delete(format!(
            "http://localhost:{port}/instances/{server_instance}"
        ))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[cfg(not(unix))]
fn shutdown_pocketic(_: u16, _: usize, _: usize, _: Logger) -> DfxResult {
    bail!("PocketIC not supported on this platform")
}
