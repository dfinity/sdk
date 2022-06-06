use crate::config::dfinity::to_socket_addr;
use crate::lib::error::DfxResult;

use fn_error_context::context;
use std::net::SocketAddr;

#[derive(Clone, Debug, PartialEq)]
pub struct LocalServerDescriptor {
    pub bind: String,
}

impl LocalServerDescriptor {
    #[context("Failed to convert {} to a bind address", self.bind)]
    pub fn bind_address(&self) -> DfxResult<SocketAddr> {
        to_socket_addr(&self.bind)
    }
}
