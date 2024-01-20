use ic_agent::AgentError;

pub(crate) fn retryable(agent_error: &AgentError) -> bool {
    matches!(
        agent_error,
        AgentError::TimeoutWaitingForResponse() | AgentError::TransportError(_)
    )
}
