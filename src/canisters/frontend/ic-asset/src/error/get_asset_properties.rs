use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetAssetPropertiesError {
    #[error("Failed to get asset properties for {0}: {1}")]
    GetAssetPropertiesFailed(String, AgentError),
}
