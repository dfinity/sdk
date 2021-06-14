use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::webserver::run_webserver;

use crate::actors::proxy_webserver_coordinator::signals::StartWebserver;
use actix::clock::{delay_for, Duration};
use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context, Handler, ResponseFuture};
use actix_server::Server;
use futures::future;
use futures::future::FutureExt;
use slog::{error, info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;

pub mod signals {
    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct StartWebserver {}
}

pub struct Config {
    pub logger: Option<Logger>,
    pub shutdown_controller: Addr<ShutdownController>,
    pub bind: SocketAddr,
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
}

///
/// The ProxyWebserverCoordinator runs a webserver for the /_/ endpoint.
///
/// It's a little more complicated that it could be, mostly in order to ensure a clean shutdown.
pub struct ProxyWebserverCoordinator {
    logger: Logger,
    config: Config,
    server: Option<Server>,
}

impl ProxyWebserverCoordinator {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        ProxyWebserverCoordinator {
            config,
            logger,
            server: None,
        }
    }

    fn start_server(&self) -> DfxResult<Server> {
        info!(self.logger, "Starting webserver for /_/");

        run_webserver(
            self.logger.clone(),
            self.config.build_output_root.clone(),
            self.config.network_descriptor.clone(),
            self.config.bind,
        )
    }
}

impl Actor for ProxyWebserverCoordinator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let _ = ctx.address().recipient().do_send(StartWebserver {});
        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }
}

impl Handler<StartWebserver> for ProxyWebserverCoordinator {
    type Result = ();

    fn handle(&mut self, msg: StartWebserver, ctx: &mut Self::Context) {
        if let Some(server) = &self.server {
            ctx.wait(wrap_future(server.stop(true)));
            self.server = None;
            ctx.address().do_send(msg);
        } else {
            match self.start_server() {
                Ok(server) => {
                    self.server = Some(server);
                }
                Err(e) => {
                    error!(self.logger, "Unable to start webserver: {}", e);
                    ctx.wait(wrap_future(delay_for(Duration::from_secs(2))));
                    ctx.address().do_send(msg);
                }
            }
        }
    }
}

impl Handler<Shutdown> for ProxyWebserverCoordinator {
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
