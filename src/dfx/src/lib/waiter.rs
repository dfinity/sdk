use delay::Delay;
use std::time::Duration;

const RETRY_PAUSE: Duration = Duration::from_millis(200);
const MAX_RETRY_PAUSE: Duration = Duration::from_secs(1);

pub fn waiter_with_timeout(duration: Duration) -> Delay {
    Delay::builder()
        .exponential_backoff_capped(RETRY_PAUSE, 1.4, MAX_RETRY_PAUSE)
        .timeout(duration)
        .build()
}
