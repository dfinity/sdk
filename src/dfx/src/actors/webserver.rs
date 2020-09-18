use crate::lib::error::{DfxError, DfxResult};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::webserver::webserver;
use actix::{Actor, Context, Running};
use actix_server::Server;
use futures::executor::block_on;
use slog::info;
use slog::Logger;
use std::net::SocketAddr;
use std::path::PathBuf;

/// The configuration for the webserver actor.
pub struct Config {
    pub logger: Option<Logger>,
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
    pub bind: SocketAddr,
    pub clients_api_uri: Vec<url::Url>,
    pub serve_dir: PathBuf,
}

impl Config {
    /// Validate the configuration.  This happens before attempting to start
    /// the actor, because if Actor.started() panics, the actor system
    /// never exits.  I'd love to know why.
    pub fn validate(self) -> DfxResult<Self> {
        // Verify that we cannot bind to a port that we forward to.
        let bound_port = self.bind.port();
        let bind_and_forward_on_same_port = self.clients_api_uri.iter().any(|url| {
            Some(bound_port) == url.port()
                && match url.host_str() {
                    Some(h) => h == "localhost" || h == "::1" || h == "127.0.0.1",
                    None => true,
                }
        });
        if bind_and_forward_on_same_port {
            Err(DfxError::Unknown(
                "Cannot forward API calls to the same bootstrap server.".to_string(),
            ))
        } else {
            Ok(self)
        }
    }
}

/// A webserver actor.
pub struct Webserver {
    logger: Logger,
    config: Config,

    server: Option<Server>,
}

impl Webserver {
    pub fn new(config: Config) -> DfxResult<Self> {
        let config = config.validate()?;
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        Ok(Webserver {
            logger,
            config,
            server: None,
        })
    }

    fn start_webserver(&mut self) -> DfxResult {
        let config = &self.config;

        let handle = webserver(
            self.logger.clone(),
            config.build_output_root.clone(),
            config.network_descriptor.clone(),
            config.bind,
            config.clients_api_uri.clone(),
            &config.serve_dir,
        )?;

        self.server = Some(handle);
        Ok(())
    }
}

impl Actor for Webserver {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        // If this .expect() panics, the process won't exit, and I don't know what to do about it.
        self.start_webserver()
            .expect("Could not start the webserver");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!(self.logger, "Stopping the webserver...");

        if let Some(server) = self.server.take() {
            block_on(server.stop(true));
        }

        info!(self.logger, "Stopped.");
        Running::Stop
    }
}
