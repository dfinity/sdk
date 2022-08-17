use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;

use anyhow::Context;
use fn_error_context::context;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Deserialize, Serialize, Debug)]
struct NetworkMetadata {
    created: OffsetDateTime,
}

#[context("Failed write network id to {}.", local_server_descriptor.network_id_path().display())]
pub fn write_network_id(local_server_descriptor: &LocalServerDescriptor) -> DfxResult {
    let contents = NetworkMetadata {
        created: OffsetDateTime::now_utc(),
    };
    let contents =
        serde_json::to_string_pretty(&contents).context("Failed to pretty-format to string")?;
    std::fs::write(local_server_descriptor.network_id_path(), contents)
        .context("Failed to write to file")?;
    Ok(())
}
