#[cfg(not(feature = "reqwest-0_12"))]
use reqwest::StatusCode;
#[cfg(feature = "reqwest-0_12")]
use reqwest012::StatusCode;

#[cfg(not(feature = "reqwest-0_12"))]
pub fn is_retryable(err: &reqwest::Error) -> bool {
    err.is_timeout()
        || err.is_connect()
        || matches!(
            err.status(),
            Some(
                StatusCode::INTERNAL_SERVER_ERROR
                    | StatusCode::BAD_GATEWAY
                    | StatusCode::SERVICE_UNAVAILABLE
                    | StatusCode::GATEWAY_TIMEOUT
                    | StatusCode::TOO_MANY_REQUESTS
            )
        )
}

#[cfg(feature = "reqwest-0_12")]
pub fn is_retryable(err: &reqwest012::Error) -> bool {
    err.is_timeout()
        || err.is_connect()
        || matches!(
            err.status(),
            Some(
                StatusCode::INTERNAL_SERVER_ERROR
                    | StatusCode::BAD_GATEWAY
                    | StatusCode::SERVICE_UNAVAILABLE
                    | StatusCode::GATEWAY_TIMEOUT
                    | StatusCode::TOO_MANY_REQUESTS
            )
        )
}
