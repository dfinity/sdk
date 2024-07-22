use ic_agent::agent::http_transport::reqwest_transport::reqwest::StatusCode;
use ic_agent::AgentError;

pub(crate) fn retryable(agent_error: &AgentError) -> bool {
    match agent_error {
        AgentError::TimeoutWaitingForResponse() => true,
        AgentError::TransportError(_) => true,
        AgentError::HttpError(http_error) => {
            http_error.status == StatusCode::INTERNAL_SERVER_ERROR
                || http_error.status == StatusCode::BAD_GATEWAY
                || http_error.status == StatusCode::SERVICE_UNAVAILABLE
                || http_error.status == StatusCode::GATEWAY_TIMEOUT
                || http_error.status == StatusCode::TOO_MANY_REQUESTS
        }
        _ => false,
    }
}
