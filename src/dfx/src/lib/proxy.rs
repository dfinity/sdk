use crate::lib::network::network_descriptor::NetworkDescriptor;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Provide basic information to the proxy about the API port, the
/// address and the serve directory.
#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub bind: SocketAddr,
    pub serve_dir: PathBuf,
    pub providers: Vec<url::Url>,
    pub logger: slog::Logger,
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
}
