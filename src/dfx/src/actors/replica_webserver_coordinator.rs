use crate::actors::replica::signals::outbound::ReplicaReadySignal;
use crate::actors::replica::signals::PortReadySubscribe;
use crate::actors::replica::Replica;
use crate::lib::error::DfxResult;
use crate::lib::proxy::ProxyConfig;
use crate::lib::webserver::run_webserver;
use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use actix_server::Server;
use slog::{info, Logger};

pub struct Config {
    pub replica_addr: Addr<Replica>,
    pub logger: Option<Logger>,
    pub proxy_config: ProxyConfig,
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
        let proxy_config = &self.config.proxy_config;
        let mut providers = proxy_config.providers.clone();

        let ic_client_bind_addr = "http://localhost:".to_owned() + port.to_string().as_str();
        let ic_client_bind_addr = ic_client_bind_addr.as_str();
        let client_api_uri =
            url::Url::parse(ic_client_bind_addr).expect("Failed to parse replica ingress url.");
        // Add the localhost as an option.
        providers.push(client_api_uri);
        eprintln!("replica address: {:?}", ic_client_bind_addr);

        let server = run_webserver(
            proxy_config.logger.clone(),
            proxy_config.build_output_root.clone(),
            proxy_config.network_descriptor.clone(),
            proxy_config.bind,
            providers,
            proxy_config.serve_dir.clone(),
        )?;

        // webserver(
        //     proxy_config.logger.clone(),
        //     proxy_config.build_output_root.clone(),
        //     proxy_config.network_descriptor.clone(),
        //     proxy_config.bind,
        //     providers,
        //     &proxy_config.serve_dir,
        //     sender,
        // )?.join()
        //     .map_err(|e| {
        //         DfxError::RuntimeError(Error::new(
        //             ErrorKind::Other,
        //             format!("Failed while running frontend proxy thead -- {:?}", e),
        //         ))
        //     })?;
        //
        // // Wait for the webserver to be started.
        // let server = receiver.recv().expect("Failed to receive server...");

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
            ctx.wait(wrap_future(server.stop(true)));
            self.server = None;
        }

        let server = self.start_server(msg.port).unwrap();
        self.server = server;
    }
}
