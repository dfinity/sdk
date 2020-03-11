use crate::lib::error::{DfxError, DfxResult};

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct HttpHandlerConfig {
    /// Instructs the HTTP handler to use the specified port
    pub use_port: Option<u16>,

    /// Instructs the HTTP handler to bind to any open port and report the port
    /// to the specified file.
    /// The port is written in its textual representation, no newline at the
    /// end.
    pub write_port_to: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub struct SchedulerConfig {
    pub exec_gas: Option<u64>,
    pub round_gas_max: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct StateManagerConfig {
    pub state_root: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct ReplicaConfig {
    pub http_handler: HttpHandlerConfig,
    pub scheduler: SchedulerConfig,
    pub state_manager: StateManagerConfig,
}

impl ReplicaConfig {
    pub fn new(state_root: &Path) -> Self {
        ReplicaConfig {
            http_handler: HttpHandlerConfig {
                write_port_to: None,
                use_port: None,
            },
            scheduler: SchedulerConfig {
                exec_gas: None,
                round_gas_max: None,
            },
            state_manager: StateManagerConfig {
                state_root: state_root.to_path_buf(),
            },
        }
    }

    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.http_handler.use_port = Some(port);
        self.http_handler.write_port_to = None;
        self
    }

    pub fn with_random_port(&mut self, write_port_to: &Path) -> &mut Self {
        self.http_handler.use_port = None;
        self.http_handler.write_port_to = Some(write_port_to.to_path_buf());
        self
    }

    pub fn with_scheduler(&mut self, scheduler: SchedulerConfig) -> &mut Self {
        self.scheduler = scheduler;
        self
    }

    pub fn to_toml(&self) -> DfxResult<String> {
        toml::to_string(&self).map_err(DfxError::CouldNotSerializeClientConfiguration)
    }
}
