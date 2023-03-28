use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchRootKeyError {
    #[error("Encountered an error while trying to query the replica: {0}")]
    ReplicaError(AgentError),

    #[error("This command only runs on local instances. Cannot run this on the real IC.")]
    NotLocal,
}
