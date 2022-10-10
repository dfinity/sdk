use garcon::Delay;
use std::time::Duration;

pub(crate) fn waiter_with_timeout(duration: Duration) -> Delay {
    Delay::builder().timeout(duration).build()
}
