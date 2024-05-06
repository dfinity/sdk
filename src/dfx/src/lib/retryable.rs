use ic_agent::AgentError;

pub fn retryable(agent_error: &AgentError) -> bool {
    matches!(
        agent_error,
        AgentError::TimeoutWaitingForResponse() | AgentError::TransportError(_)
    )
}
