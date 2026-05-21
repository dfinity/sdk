use backoff::SystemClock;
use backoff::exponential::ExponentialBackoff;
use backoff::future::retry;
use url::Url;

#[cfg(not(feature = "reqwest-0_12"))]
pub async fn get_with_retries(
    url: Url,
    retry_policy: ExponentialBackoff<SystemClock>,
) -> Result<reqwest::Response, reqwest::Error> {
    let operation = || async {
        let response = reqwest::get(url.clone())
            .await
            .and_then(|resp| resp.error_for_status());
        match response {
            Ok(doc) => Ok(doc),
            Err(e) if crate::error::reqwest::is_retryable(&e) => Err(backoff::Error::transient(e)),
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    };
    retry(retry_policy, operation).await
}

#[cfg(feature = "reqwest-0_12")]
pub async fn get_with_retries(
    url: Url,
    retry_policy: ExponentialBackoff<SystemClock>,
) -> Result<reqwest012::Response, reqwest012::Error> {
    let operation = || async {
        let response = reqwest012::get(url.clone())
            .await
            .and_then(|resp| resp.error_for_status());
        match response {
            Ok(doc) => Ok(doc),
            Err(e) if crate::error::reqwest::is_retryable(&e) => Err(backoff::Error::transient(e)),
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    };
    retry(retry_policy, operation).await
}
