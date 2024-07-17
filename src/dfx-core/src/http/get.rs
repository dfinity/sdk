use backoff::exponential::ExponentialBackoff;
use backoff::future::retry;
use backoff::SystemClock;
use reqwest::Response;
use url::Url;

pub async fn get_with_retries(
    url: Url,
    retry_policy: ExponentialBackoff<SystemClock>,
) -> Result<Response, reqwest::Error> {
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
