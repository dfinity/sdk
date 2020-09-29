use crate::actors::replica::signals::outbound::ReplicaReadySignal;
use crate::actors::replica::signals::PortReadySubscribe;
use crate::actors::replica::Replica;
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::webserver::run_webserver;
use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use actix_server::Server;
use slog::{info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;
use actix::clock::{Duration, delay_for};

pub struct Config {
    pub logger: Option<Logger>,
    pub replica_addr: Addr<Replica>,
    pub bind: SocketAddr,
    pub serve_dir: PathBuf,
    pub providers: Vec<url::Url>,
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
}

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

    fn start_server(&self, port: u16) -> DfxResult<Option<Server>> {
        let mut providers = self.config.providers.clone();

        let ic_client_bind_addr = "http://localhost:".to_owned() + port.to_string().as_str();
        let ic_client_bind_addr = ic_client_bind_addr.as_str();
        let client_api_uri =
            url::Url::parse(ic_client_bind_addr).expect("Failed to parse replica ingress url.");
        // Add the localhost as an option.
        providers.push(client_api_uri);
        eprintln!("replica address: {:?}", ic_client_bind_addr);

        let server = run_webserver(
            self.logger.clone(),
            self.config.build_output_root.clone(),
            self.config.network_descriptor.clone(),
            self.config.bind,
            providers,
            self.config.serve_dir.clone(),
        )?;

        Ok(Some(server))
    }
}

impl Actor for ReplicaWebserverCoordinator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(self.logger, "ReplicaWebserverCoordinator started");
        self.config
            .replica_addr
            .do_send(PortReadySubscribe(ctx.address().recipient()));
    }
}

impl Handler<ReplicaReadySignal> for ReplicaWebserverCoordinator {
    type Result = ();

    fn handle(&mut self, msg: ReplicaReadySignal, ctx: &mut Self::Context) {
        info!(
            self.logger,
            "ReplicaWebserverCoordinator: replica ready on {}", msg.port
        );
        println!("replica ready {}", msg.port);

        if let Some(server) = &self.server {
            println!("stopping webserver");
            ctx.wait(wrap_future(server.stop(true)));
            self.server = None;
            println!("delay before restarting webserver");
            ctx.wait(wrap_future(delay_for(Duration::from_secs(10))));
            ctx.address().do_send(ReplicaReadySignal { port: msg.port });
        }
        else {
            println!("starting webserver");

            let server = self.start_server(msg.port).unwrap();
            self.server = server;
        }

    }
}
