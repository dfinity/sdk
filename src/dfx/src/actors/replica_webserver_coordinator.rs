use crate::actors::replica::signals::outbound::ReplicaReadySignal;
use crate::actors::replica::signals::PortReadySubscribe;
use crate::actors::replica::Replica;
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::webserver::run_webserver;

use actix::clock::{delay_for, Duration};
use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context, Handler, ResponseFuture};
use actix_server::Server;
use futures::future;
use futures::future::FutureExt;
use slog::{debug, error, info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Config {
    pub logger: Option<Logger>,
    pub replica_addr: Addr<Replica>,
    pub shutdown_controller: Addr<ShutdownController>,
    pub bind: SocketAddr,
    pub serve_dir: PathBuf,
    pub providers: Vec<url::Url>,
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
}

///
/// The ReplicaWebserverCoordinator runs a webserver for the replica.
///
/// If the replica restarts, it will start a new webserver for the new replica.
pub struct ReplicaWebserverCoordinator {
    logger: Logger,
    config: Config,
    server: Option<Server>,
}

impl ReplicaWebserverCoordinator {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        ReplicaWebserverCoordinator {
            config,
            logger,
            server: None,
        }
    }

    fn start_server(&self, port: u16) -> DfxResult<Server> {
        let mut providers = self.config.providers.clone();

        let ic_replica_bind_addr = "http://localhost:".to_owned() + port.to_string().as_str();
        let ic_replica_bind_addr = ic_replica_bind_addr.as_str();
        let replica_api_uri =
            url::Url::parse(ic_replica_bind_addr).expect("Failed to parse replica ingress url.");
        providers.push(replica_api_uri);
        info!(
            self.logger,
            "Starting webserver on port {} for replica at {:?}", port, ic_replica_bind_addr
        );

        run_webserver(
            self.logger.clone(),
            self.config.build_output_root.clone(),
            self.config.network_descriptor.clone(),
            self.config.bind,
            providers,
            self.config.serve_dir.clone(),
        )
    }
}

impl Actor for ReplicaWebserverCoordinator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.config
            .replica_addr
            .do_send(PortReadySubscribe(ctx.address().recipient()));
        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }
}

impl Handler<ReplicaReadySignal> for ReplicaWebserverCoordinator {
    type Result = ();

    fn handle(&mut self, msg: ReplicaReadySignal, ctx: &mut Self::Context) {
        debug!(self.logger, "replica ready on {}", msg.port);

        if let Some(server) = &self.server {
            ctx.wait(wrap_future(server.stop(true)));
            self.server = None;
            ctx.address().do_send(msg);
        } else {
            match self.start_server(msg.port) {
                Ok(server) => {
                    self.server = Some(server);
                }
                Err(e) => {
                    error!(
                        self.logger,
                        "Unable to start webserver on port {}: {}", msg.port, e
                    );
                    ctx.wait(wrap_future(delay_for(Duration::from_secs(2))));
                    ctx.address().do_send(msg);
                }
            }
        }
    }
}

impl Handler<Shutdown> for ReplicaWebserverCoordinator {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, _msg: Shutdown, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(server) = self.server.take() {
            // We stop the webserver before shutting down because
            // if we don't, the process will segfault
            // while dropping actix stuff after main() returns.
            Box::pin(server.stop(true).map(Ok))
        } else {
            Box::pin(future::ok(()))
        }
    }
}
