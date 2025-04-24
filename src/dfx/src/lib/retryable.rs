use ic_agent::{agent::OperationStatus, AgentError};
use ic_utils::error::{BaseError, CanisterError};

pub fn retryable(agent_error: &AgentError) -> bool {
    agent_error
        .operation_info()
        .is_some_and(|op| op.status == OperationStatus::NotSent)
}

pub fn canister_retryable(canister_error: &BaseError) -> bool {
    canister_error.as_agent().is_some_and(retryable)
}
