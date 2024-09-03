use crate::actors::btc_adapter::signals::{BtcAdapterReady, BtcAdapterReadySubscribe};
use crate::actors::canister_http_adapter::signals::{
    CanisterHttpAdapterReady, CanisterHttpAdapterReadySubscribe,
};
use crate::actors::pocketic_proxy::signals::{PortReadySignal, PortReadySubscribe};
use crate::actors::replica::signals::ReplicaRestarted;
use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::integrations::bitcoin::initialize_bitcoin_canister;
use crate::lib::integrations::create_integrations_agent;
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Recipient,
    ResponseActFuture, Running, WrapFuture,
};
use anyhow::bail;
use crossbeam::channel::{unbounded, Receiver, Sender};
use dfx_core::config::model::replica_config::ReplicaConfig;
use slog::{debug, error, info, Logger};
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

#[derive(Clone)]
pub struct BitcoinIntegrationConfig {
    pub canister_init_arg: String,
}

/// The configuration for the replica actor.
pub struct Config {
    pub ic_starter_path: PathBuf,
    pub replica_config: ReplicaConfig,
    pub bitcoin_integration_config: Option<BitcoinIntegrationConfig>,
    pub replica_path: PathBuf,
    pub replica_pid_path: PathBuf,
    pub shutdown_controller: Addr<ShutdownController>,
    pub logger: Option<Logger>,
    pub btc_adapter_ready_subscribe: Option<Recipient<BtcAdapterReadySubscribe>>,
    pub canister_http_adapter_ready_subscribe: Option<Recipient<CanisterHttpAdapterReadySubscribe>>,
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

    // We must wait until certain other actors are ready, if they are enabled
    awaiting_btc_adapter_ready: bool,
    awaiting_canister_http_adapter_ready: bool,
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
            awaiting_btc_adapter_ready: false,
            awaiting_canister_http_adapter_ready: false,
            logger,
        }
    }

    /// Wait for `ic-starter` process writing the http port file.
    /// Retry every 0.1s for 2 minutes.
    /// Will break out of the loop if receive stop signal.
    ///
    /// Returns
    /// - Ok(Some(port)) if succeed;
    /// - Ok(None) if receive stop signal (`dfx start` then Ctrl-C immediately);
    /// - Err if time out;
    fn wait_for_port_file(
        file_path: &Path,
        stop_receiver: &Receiver<()>,
    ) -> DfxResult<Option<u16>> {
        let mut retries = 0;
        loop {
            if stop_receiver.try_recv().is_ok() {
                return Ok(None);
            }
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if let Ok(port) = content.parse::<u16>() {
                    return Ok(Some(port));
                }
            }
            if retries >= 1200 {
                bail!("Cannot start the replica: timed out");
            }
            std::thread::sleep(Duration::from_millis(100));
            retries += 1;
        }
    }

    fn restart_replica_if_all_ready(&mut self, addr: Addr<Self>) {
        let done_waiting =
            !self.awaiting_canister_http_adapter_ready && !self.awaiting_btc_adapter_ready;
        if done_waiting {
            self.stop_replica();
            self.start_replica(addr)
                .expect("unable to start the replica");
        }
    }

    fn start_replica(&mut self, addr: Addr<Self>) -> DfxResult {
        let logger = self.logger.clone();

        // Create a replica config.
        let config = &self.config.replica_config;
        let replica_pid_path = self.config.replica_pid_path.to_path_buf();

        let port = config.http_handler.port;
        let write_port_to = config.http_handler.write_port_to.clone();
        let artificial_delay = config.artificial_delay;
        let replica_path = self.config.replica_path.to_path_buf();
        let ic_starter_path = self.config.ic_starter_path.to_path_buf();

        let (sender, receiver) = unbounded();

        let handle = anyhow::Context::context(
            replica_start_thread(
                logger,
                config.clone(),
                self.config.bitcoin_integration_config.clone(),
                port,
                write_port_to,
                ic_starter_path,
                replica_path,
                replica_pid_path,
                artificial_delay,
                addr,
                receiver,
            ),
            "Failed to start replica thread.",
        )?;

        self.thread_join = Some(handle);
        self.stop_sender = Some(sender);
        Ok(())
    }

    fn stop_replica(&mut self) {
        if self.stop_sender.is_some() || self.thread_join.is_some() {
            debug!(self.logger, "stopping replica");
        }

        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(join) = self.thread_join.take() {
            let _ = join.join();
        }
    }

    fn send_ready_signal(&self, port: u16) {
        for sub in &self.ready_subscribers {
            sub.do_send(PortReadySignal {
                url: format!("http://localhost:{port}"),
            });
        }
    }
}

impl Actor for Replica {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(btc_adapter_ready_subscribe) = &self.config.btc_adapter_ready_subscribe {
            btc_adapter_ready_subscribe
                .do_send(BtcAdapterReadySubscribe(ctx.address().recipient()));
            self.awaiting_btc_adapter_ready = true;
        }
        if let Some(subscribe) = &self.config.canister_http_adapter_ready_subscribe {
            subscribe.do_send(CanisterHttpAdapterReadySubscribe(ctx.address().recipient()));
            self.awaiting_canister_http_adapter_ready = true;
        }

        self.restart_replica_if_all_ready(ctx.address());

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(self.logger, "Stopping the replica...");
        self.stop_replica();

        info!(self.logger, "Stopped.");
        Running::Stop
    }
}

impl Handler<PortReadySubscribe> for Replica {
    type Result = ();

    fn handle(&mut self, msg: PortReadySubscribe, _: &mut Self::Context) {
        // If we have a port, send that we're already ready! Yeah!
        if let Some(port) = self.port {
            msg.0.do_send(PortReadySignal {
                url: format!("http://localhost:{port}"),
            });
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

impl Handler<BtcAdapterReady> for Replica {
    type Result = ();

    fn handle(&mut self, _msg: BtcAdapterReady, ctx: &mut Self::Context) {
        debug!(self.logger, "btc adapter ready");
        self.awaiting_btc_adapter_ready = false;

        self.restart_replica_if_all_ready(ctx.address());
    }
}

impl Handler<CanisterHttpAdapterReady> for Replica {
    type Result = ();

    fn handle(&mut self, _msg: CanisterHttpAdapterReady, ctx: &mut Self::Context) {
        debug!(self.logger, "canister http adapter ready");
        self.awaiting_canister_http_adapter_ready = false;

        self.restart_replica_if_all_ready(ctx.address());
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

fn replica_start_thread(
    logger: Logger,
    config: ReplicaConfig,
    bitcoin_integration_config: Option<BitcoinIntegrationConfig>,
    port: Option<u16>,
    write_port_to: Option<PathBuf>,
    ic_starter_path: PathBuf,
    replica_path: PathBuf,
    replica_pid_path: PathBuf,
    artificial_delay: u32,
    addr: Addr<Replica>,
    receiver: Receiver<()>,
) -> DfxResult<std::thread::JoinHandle<()>> {
    let thread_handler = move || {
        // Start the process, then wait for the file.
        let ic_starter_path = ic_starter_path.as_os_str();

        // form the ic-start command here similar to replica command
        let mut cmd = std::process::Command::new(ic_starter_path);
        cmd.args([
            "--replica-path",
            replica_path.to_str().unwrap_or_default(),
            "--state-dir",
            config.state_manager.state_root.to_str().unwrap_or_default(),
            "--create-funds-whitelist",
            "*",
            "--subnet-type",
            &config.subnet_type.as_ic_starter_string(),
            "--chain-key-ids",
            "ecdsa:Secp256k1:dfx_test_key",
            "--chain-key-ids",
            "schnorr:Bip340Secp256k1:dfx_test_key",
            "--chain-key-ids",
            "schnorr:Ed25519:dfx_test_key",
            "--log-level",
            &config.log_level.as_ic_starter_string(),
            "--use-specified-ids-allocation-range",
        ]);
        #[cfg(target_os = "macos")]
        cmd.args(["--consensus-pool-backend", "rocksdb"]);
        if let Some(port) = port {
            cmd.args(["--http-port", &port.to_string()]);
        }
        // Enable canister sandboxing to be consistent with the mainnet.
        // The flag will be removed on the `ic-starter` side once this
        // change is rolled out without any issues.
        cmd.args(["--subnet-features", "canister_sandboxing"]);
        if config.btc_adapter.enabled {
            if let Some(socket_path) = config.btc_adapter.socket_path {
                cmd.args([
                    "--bitcoin-testnet-uds-path",
                    socket_path.to_str().unwrap_or_default(),
                ]);
            }
        }
        if config.canister_http_adapter.enabled {
            cmd.args(["--subnet-features", "http_requests"]);
            if let Some(socket_path) = config.canister_http_adapter.socket_path {
                cmd.args([
                    "--canister-http-uds-path",
                    socket_path.to_str().unwrap_or_default(),
                ]);
            }
        }

        if let Some(write_port_to) = &write_port_to {
            cmd.args(["--http-port-file", &write_port_to.to_string_lossy()]);
        }
        cmd.args([
            "--initial-notary-delay-millis",
            // The initial notary delay is set to 2500ms in the replica's
            // default subnet configuration to help running tests.
            // For our production network, we actually set them to 600ms.
            &format!("{artificial_delay}"),
        ]);

        // This should agree with the value at
        // at https://gitlab.com/dfinity-lab/core/ic/-/blob/master/ic-os/guestos/rootfs/etc/systemd/system/ic-replica.service
        cmd.env("RUST_MIN_STACK", "8192000");

        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        loop {
            if let Some(port_path) = write_port_to.as_ref() {
                let _ = std::fs::remove_file(port_path);
            }
            let last_start = std::time::Instant::now();
            debug!(logger, "Starting replica...");
            let mut child = cmd.spawn().expect("Could not start replica.");

            std::fs::write(&replica_pid_path, "").expect("Could not write to replica-pid file.");
            std::fs::write(&replica_pid_path, child.id().to_string())
                .expect("Could not write to replica-pid file.");

            let port = if let Some(p) = port {
                p
            } else {
                match Replica::wait_for_port_file(write_port_to.as_ref().unwrap(), &receiver)
                    .unwrap()
                {
                    Some(p) => p,
                    // If Ctrl-C right after `dfx start`, the `ic-starter` child process will be killed already.
                    // And the `write_port_to` file will never be ready.
                    // So we let `wait_for_port_file` method to break out from the waiting,
                    // finish this actor starting ASAP and let the system stop the actor.
                    None => break,
                }
            };

            if let Err(e) =
                initialize_replica(port, logger.clone(), bitcoin_integration_config.clone())
            {
                error!(logger, "Failed to initialize replica: {:#}", e);
                let _ = child.kill();
                let _ = child.wait();
                if receiver.try_recv().is_ok() {
                    debug!(logger, "Got signal to stop.");
                    break;
                } else {
                    continue;
                }
            }
            addr.do_send(signals::ReplicaRestarted { port });
            let log_clone = logger.clone();
            debug!(log_clone, "Dashboard: http://localhost:{port}/_/dashboard");

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the replica if done = true.
            match wait_for_child_or_receiver(&mut child, &receiver) {
                ChildOrReceiver::Receiver => {
                    debug!(logger, "Got signal to stop. Killing replica process...");
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                ChildOrReceiver::Child => {
                    debug!(logger, "Replica process failed.");
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
        .name("replica-actor".to_owned())
        .spawn(thread_handler)
        .map_err(DfxError::from)
}

#[tokio::main(flavor = "current_thread")]
async fn initialize_replica(
    port: u16,
    logger: Logger,
    bitcoin_integration_config: Option<BitcoinIntegrationConfig>,
) -> DfxResult {
    let agent_url = format!("http://localhost:{port}");

    debug!(logger, "Waiting for replica to report healthy status");
    crate::lib::replica::status::ping_and_wait(&agent_url).await?;

    let agent = create_integrations_agent(&agent_url, &logger).await?;

    if let Some(bitcoin_integration_config) = bitcoin_integration_config {
        initialize_bitcoin_canister(&agent, &logger, bitcoin_integration_config).await?;
    }

    info!(logger, "Initialized replica.");

    Ok(())
}
