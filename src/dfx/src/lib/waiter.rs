use delay::Delay;
use std::time::Duration;

const RETRY_PAUSE: Duration = Duration::from_millis(200);
const MAX_RETRY_PAUSE: Duration = Duration::from_secs(1);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

pub fn create_waiter() -> Delay {
    Delay::builder()
        .exponential_backoff_capped(RETRY_PAUSE, 1.4, MAX_RETRY_PAUSE)
        .timeout(REQUEST_TIMEOUT)
        .build()
}
