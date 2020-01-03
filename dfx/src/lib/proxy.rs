use crate::lib::webserver::run_webserver;
use actix_server::Server;
use crossbeam::channel::{Receiver, Sender};
use futures::future::Future;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Proxy {
    config: ProxyConfig,
    server_handle: ProxyServer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProxyConfig {
    pub client_api_port: String,
    pub bind: SocketAddr,
    pub serve_dir: PathBuf,
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

    // Shutdown and start are private (for now). The cost of inlining
    // is minimal, while as the logic gets more complicated inlining
    // them in restart function has the possibility to reduce terms as
    // rustc gets better.
    #[inline(always)]
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
    #[inline(always)]
    pub fn start(self, sender: Sender<Server>, receiver: Receiver<Server>) -> Result<Self> {
        run_webserver(
            self.config.bind,
            self.config.client_api_port.clone(),
            self.config.serve_dir.clone(),
            sender.clone(),
        )
        .expect("Failed to start webserver.");

        let mut new_server = Proxy::new(self.config);
        let handle = ServerHandle { sender, receiver };
        new_server.server_handle = ProxyServer::Up(handle);
        Ok(new_server)
    }

    #[inline(always)]
    pub fn set_client_api_port(self, client_api_port: String) -> Self {
        let mut handle = self;
        handle.config.client_api_port = client_api_port;
        handle
    }

    /// Restart a proxy with a new configuration.
    //
    pub fn restart(self, sender: Sender<Server>, receiver: Receiver<Server>) -> Result<Self> {
        let config = self.config.clone();
        let mut handle = self.shutdown()?;
        handle.config = config;
        handle.start(sender, receiver)
    }

    /// Return proxy client api port.
    pub fn port(&self) -> String {
        self.config.client_api_port.clone()
    }
}
