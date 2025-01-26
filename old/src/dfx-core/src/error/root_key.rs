use ic_agent;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchRootKeyError {
    #[error(transparent)]
    AgentError(#[from] ic_agent::AgentError),

    #[error("This command only runs on local instances. Cannot run this on the real IC.")]
    MustNotFetchRootKeyOnMainnet,
}
