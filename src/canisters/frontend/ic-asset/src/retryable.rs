use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::AgentError;

pub(crate) fn retryable(agent_error: &AgentError) -> bool {
    match agent_error {
        AgentError::ReplicaError {
            reject_code,
            reject_message,
        } if *reject_code == 5 && reject_message.contains("is out of cycles") => false,
        AgentError::ReplicaError {
            reject_code,
            reject_message,
        } if *reject_code == 5 && reject_message.contains("Fail to decode") => false,
        AgentError::ReplicaError {
            reject_code,
            reject_message,
        } if *reject_code == 4 && reject_message.contains("is not authorized") => false,
        AgentError::HttpError(HttpErrorPayload {
            status,
            content_type: _,
            content: _,
        }) if *status == 403 => {
            // sometimes out of cycles looks like this
            // assume any 403(unauthorized) is not retryable
            false
        }
        _ => true,
    }
}
