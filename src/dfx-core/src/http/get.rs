use crate::error::reqwest::WrappedReqwestError;
use crate::http::retryable::Retryable;
use backoff::exponential::ExponentialBackoff;
use backoff::future::retry;
use backoff::SystemClock;
use reqwest::Response;
use url::Url;

pub async fn get_with_retries(
    url: Url,
    retry_policy: ExponentialBackoff<SystemClock>,
) -> Result<Response, WrappedReqwestError> {
    let operation = || async {
        let response = reqwest::get(url.clone())
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(WrappedReqwestError);
        match response {
            Ok(doc) => Ok(doc),
            Err(e) if e.is_retryable() => Err(backoff::Error::transient(e)),
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    };
    retry(retry_policy, operation).await
}
