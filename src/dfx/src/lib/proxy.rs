use crate::lib::network::network_descriptor::NetworkDescriptor;
use actix_server::Server;
use crossbeam::channel::{Receiver, Sender};
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
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
}

#[derive(Clone, Debug)]
enum ProxyServer {}

#[derive(Clone, Debug)]
struct ServerHandle {
    sender: Sender<Server>,
    receiver: Receiver<Server>,
}
