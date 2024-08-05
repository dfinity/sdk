use crate::lib::error::DfxResult;
use anyhow::Context;
use dfx_core::config::model::local_server_descriptor::LocalServerDescriptor;
use fn_error_context::context;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Deserialize, Serialize, Debug)]
struct NetworkMetadata {
    created: OffsetDateTime,
    settings_digest: String,
}

#[context("Failed write network id to {}.", local_server_descriptor.network_id_path().display())]
pub fn write_network_id(local_server_descriptor: &LocalServerDescriptor) -> DfxResult {
    let settings_digest = local_server_descriptor
        .settings_digest()
        .to_string();
    let contents = NetworkMetadata {
        created: OffsetDateTime::now_utc(),
        settings_digest,
    };
    let contents =
        serde_json::to_string_pretty(&contents).context("Failed to pretty-format to string")?;
    std::fs::write(local_server_descriptor.network_id_path(), contents)
        .context("Failed to write to file")?;
    Ok(())
}
