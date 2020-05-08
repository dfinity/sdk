use crate::lib::webserver::run_webserver;
use actix_server::Server;
use crossbeam::channel::{Receiver, Sender};
use futures::future::Future;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;

/// A proxy that forwards requests from the browser to the network.
#[derive(Clone, Debug)]
pub struct Proxy {
    config: ProxyConfig,
    server_handle: ProxyServer,
}

/// Provide basic information to the proxy about the API port, the
/// address and the serve directory.
#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub client_api_port: u16,
    pub bind: SocketAddr,
    pub serve_dir: PathBuf,
    pub providers: Vec<url::Url>,
    pub logger: slog::Logger,
}

#[derive(Clone, Debug)]
enum ProxyServer {
    Down,
    Up(ServerHandle),
}

#[derive(Clone, Debug)]
struct ServerHandle {
    sender: Sender<Server>,
    receiver: Receiver<Server>,
}

impl Proxy {
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            server_handle: ProxyServer::Down,
        }
    }

    // Shutdown and start are private (for now).
    fn shutdown(self) -> Result<Self> {
        match self.server_handle {
            // In case the server is down we recall new() as in the
            // future we might add more bookkeeping logic, which will
            // end up in bugs. This makes this more readable to on
            // what we want. The compiler can optimize this away.
            ProxyServer::Down => Ok(Proxy::new(self.config)),
            ProxyServer::Up(handler) => {
                handler
                    .receiver
                    .try_recv()
                    .or_else(|e| {
                        Err(Error::new(
                            ErrorKind::Other,
                            format!("Failed to shutdown proxy -- {:?}", e),
                        ))
                    })?
                    .stop(true)
                    .wait()
                    .map_err(|e| {
                        Error::new(ErrorKind::Other, format!("Failed to stop server: {:?}", e))
                    })?;
                Ok(Self {
                    config: self.config,
                    server_handle: ProxyServer::Down,
                })
            }
        }
    }

    /// Start a proxy with the provided configuration. Returns a proxy
    /// handle.  Can fail to return a new proxy.
    /// # Panics
    /// Currently, we panic if the underlying webserver does not start.
    pub fn start(self, sender: Sender<Server>, receiver: Receiver<Server>) -> Result<Self> {
        let mut providers = self.config.providers.clone();

        let ic_client_bind_addr = "http://localhost:".to_owned() + self.port().to_string().as_str();
        let ic_client_bind_addr = ic_client_bind_addr.as_str();
        let client_api_uri =
            url::Url::parse(ic_client_bind_addr).expect("Failed to parse replica ingress url.");
        // Add the localhost as an option.
        providers.push(client_api_uri);
        eprintln!("replica address: {:?}", ic_client_bind_addr);

        run_webserver(
            self.config.logger.clone(),
            self.config.bind,
            providers,
            self.config.serve_dir.clone(),
            sender.clone(),
        )?;

        let mut new_server = Proxy::new(self.config);
        let handle = ServerHandle { sender, receiver };
        new_server.server_handle = ProxyServer::Up(handle);
        Ok(new_server)
    }

    /// Set the api port used by the replica. Returns a new proxy
    /// object, but does not restart the proxy.
    pub fn set_client_api_port(self, client_api_port: u16) -> Self {
        let mut handle = self;
        handle.config.client_api_port = client_api_port;
        handle
    }

    /// Restart a proxy with a new configuration.
    pub fn restart(self, sender: Sender<Server>, receiver: Receiver<Server>) -> Result<Self> {
        let config = self.config.clone();
        let mut handle = self.shutdown()?;
        handle.config = config;
        handle.start(sender, receiver)
    }

    /// Return proxy client api port.
    fn port(&self) -> u16 {
        self.config.client_api_port
    }
}

/// Supervise a Proxy.
// This should be used to refactor and simplify handling of both proxy
// and replica.
pub struct CoordinateProxy {
    pub inform_parent: Sender<Server>,
    pub server_receiver: Receiver<Server>,
    pub rcv_wait_fwatcher: Receiver<()>,
    pub request_stop_echo: Sender<()>,
    pub is_killed: Receiver<()>,
}
