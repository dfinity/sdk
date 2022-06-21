use crate::config::dfinity::to_socket_addr;
use crate::lib::error::DfxResult;

use anyhow::Context;
use fn_error_context::context;
use std::net::SocketAddr;

#[derive(Clone, Debug, PartialEq)]
pub struct LocalServerDescriptor {
    pub bind_address: SocketAddr,
}

impl LocalServerDescriptor {
    #[context("Failed to construct local server descriptor.")]
    pub(crate) fn new(bind: String) -> DfxResult<Self> {
        let bind_address =
            to_socket_addr(&bind).context("Failed to convert 'bind' field to a SocketAddress")?;
        Ok(LocalServerDescriptor { bind_address })
    }
}
