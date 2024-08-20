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
use anyhow::{anyhow, bail};
use candid::Principal;
use crossbeam::channel::{unbounded, Receiver, Sender};
use dfx_core::config::model::replica_config::ReplicaConfig;
use slog::{debug, error, info, warn, Logger};
use std::ops::ControlFlow::{self, *};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tempfile::tempdir;

pub mod signals {
    use actix::prelude::*;

    /// A message sent to the PocketIc when the process is restarted. Since we're
    /// restarting inside our own actor, this message should not be exposed.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct PocketIcRestarted {
        pub port: u16,
    }
}

pub const POCKETIC_EFFECTIVE_CANISTER_ID: Principal =
    Principal::from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x01, 0x01]);

/// The configuration for the PocketIC actor.
#[derive(Clone)]
pub struct Config {
    pub pocketic_path: PathBuf,
    pub replica_config: ReplicaConfig,
    pub port: Option<u16>,
    pub port_file: PathBuf,
    pub pid_file: PathBuf,
    pub shutdown_controller: Addr<ShutdownController>,
    pub logger: Option<Logger>,
    pub verbose: bool,
}

/// A PocketIC actor. Starts the server, can subscribe to a Ready signal and a
/// Killed signal.
/// This starts a thread that monitors the process and send signals to any subscriber
/// listening for restarts. The message contains the port the server is listening to.
///
/// Signals
///   - PortReadySubscribe
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
    ready_subscribers: Vec<Recipient<PortReadySignal>>,
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
        ready_file_path: &Path,
        shutdown_signal: Receiver<()>,
    ) -> Result<u16, ControlFlow<(), DfxError>> {
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

    fn send_ready_signal(&self, port: u16) {
        for sub in &self.ready_subscribers {
            sub.do_send(PortReadySignal {
                url: format!("http://localhost:{port}/instances/0/"),
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
        info!(self.logger, "Stopping PocketIC...");
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

impl Handler<PortReadySubscribe> for PocketIc {
    type Result = ();

    fn handle(&mut self, msg: PortReadySubscribe, _: &mut Self::Context) {
        // If we have a port, send that we're already ready! Yeah!
        if let Some(port) = self.port {
            msg.0.do_send(PortReadySignal {
                url: format!("http://localhost:{port}/instances/0/"),
            });
        }

        self.ready_subscribers.push(msg.0);
    }
}

impl Handler<signals::PocketIcRestarted> for PocketIc {
    type Result = ();

    fn handle(
        &mut self,
        msg: signals::PocketIcRestarted,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.port = Some(msg.port);
        self.send_ready_signal(msg.port);
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
            let tmp = tempdir().expect("Could not create temporary directory.");
            let ready_file = tmp.path().join("ready");
            cmd.args(["--ready-file".as_ref(), ready_file.as_os_str()]);
            if !config.verbose {
                cmd.env("RUST_LOG", "error");
            }
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            let _ = std::fs::remove_file(&config.port_file);
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting PocketIC...");
            let mut child = cmd.spawn().expect("Could not start PocketIC.");
            if let Err(e) = std::fs::write(&config.pid_file, child.id().to_string()) {
                warn!(
                    logger,
                    "Failed to write PocketIC PID to {}: {e}",
                    config.pid_file.display()
                );
            }

            let port =
                match PocketIc::wait_for_ready(&config.port_file, &ready_file, receiver.clone()) {
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
            let instance = match initialize_pocketic(
                port,
                config.replica_config.state_manager.state_root.clone(),
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
            addr.do_send(signals::PocketIcRestarted { port });
            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the server if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(logger, "Got signal to stop. Killing PocketIC process...");
                    if let Err(e) = shutdown_pocketic(port, instance, logger.clone()) {
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

#[cfg(unix)]
#[tokio::main(flavor = "current_thread")]
async fn initialize_pocketic(port: u16, state_dir: PathBuf, logger: Logger) -> DfxResult<usize> {
    use pocket_ic::common::rest::{
        CreateInstanceResponse, ExtendedSubnetConfigSet, InstanceConfig, RawTime, SubnetSpec,
    };
    use reqwest::Client;
    use time::OffsetDateTime;
    let init_client = Client::new();
    debug!(logger, "Configuring PocketIC server");
    let resp = init_client
        .post(format!("http://localhost:{port}/instances"))
        .json(&InstanceConfig {
            subnet_config_set: ExtendedSubnetConfigSet {
                nns: Some(SubnetSpec::default()),
                sns: Some(SubnetSpec::default()),
                ii: None,
                fiduciary: None,
                bitcoin: None,
                system: vec![],
                application: vec![SubnetSpec::default()],
            },
            state_dir: Some(state_dir),
            nonmainnet_features: true,
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
        CreateInstanceResponse::Created { instance_id, .. } => instance_id,
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
        .send()
        .await?
        .error_for_status()?;
    info!(logger, "Initialized PocketIC.");
    Ok(instance)
}

#[cfg(not(unix))]
fn initialize_pocketic(_: u16, _: PathBuf, _: Logger) -> DfxResult<usize> {
    bail!("PocketIC not supported on this platform")
}

#[cfg(unix)]
#[tokio::main(flavor = "current_thread")]
async fn shutdown_pocketic(port: u16, instance: usize, logger: Logger) -> DfxResult {
    use reqwest::Client;
    let shutdown_client = Client::new();
    debug!(logger, "Sending shutdown request to PocketIC server");
    shutdown_client
        .delete(format!("http://localhost:{port}/instances/{instance}"))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[cfg(not(unix))]
fn shutdown_pocketic(_: u16, _: usize, _: Logger) -> DfxResult {
    bail!("PocketIC not supported on this platform")
}
