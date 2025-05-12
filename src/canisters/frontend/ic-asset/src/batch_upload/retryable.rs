use ic_agent::agent::OperationStatus;
use ic_agent::export::reqwest::StatusCode;
use ic_utils::error::{BaseError, CanisterError};

pub(crate) fn retryable(canister_error: &BaseError) -> bool {
    let Some(agent_error) = canister_error.as_agent() else {
        return false;
    };
    if agent_error
        .operation_info()
        .is_some_and(|op| op.status == OperationStatus::NotSent)
    {
        true
    } else if let Some(http_error) = agent_error.as_http_error() {
        http_error.status == StatusCode::INTERNAL_SERVER_ERROR
            || http_error.status == StatusCode::BAD_GATEWAY
            || http_error.status == StatusCode::SERVICE_UNAVAILABLE
            || http_error.status == StatusCode::GATEWAY_TIMEOUT
            || http_error.status == StatusCode::TOO_MANY_REQUESTS
    } else {
        false
    }
}
