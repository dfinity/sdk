use ic_agent::AgentError;
use thiserror::Error;

/// Errors related to getting asset properties.
#[derive(Error, Debug)]
pub enum GetAssetPropertiesError {
    /// The call to get_asset_properties failed.
    #[error("Failed to get asset properties for {0}: {1}")]
    GetAssetPropertiesFailed(String, AgentError),
}
