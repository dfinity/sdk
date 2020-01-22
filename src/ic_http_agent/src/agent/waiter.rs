use crate::agent::agent_error::AgentError;
use std::time::{Duration, Instant};

pub trait WaiterTrait {
    fn start(&mut self) {}
    fn wait(&self) -> Result<(), AgentError>;
}

pub struct Waiter {
    inner: Box<dyn WaiterTrait>,
}

impl Waiter {
    pub fn throttle_and_timeout(throttle: Duration, timeout: Duration) -> Self {
        Waiter {
            inner: Box::new(TimeoutThrottleWaiter::new(timeout, throttle)),
        }
    }

    pub fn start(&mut self) {
        self.inner.start()
    }
    pub fn wait(&self) -> Result<(), AgentError> {
        self.inner.wait()
    }
}

impl WaiterTrait for Waiter {
    fn start(&mut self) {
        self.inner.start()
    }
    fn wait(&self) -> Result<(), AgentError> {
        self.inner.wait()
    }
}

pub struct TimeoutThrottleWaiter {
    timeout: Duration,
    start: Instant,
    throttle: Duration,
}

impl TimeoutThrottleWaiter {
    pub fn new(timeout: Duration, throttle: Duration) -> Self {
        TimeoutThrottleWaiter {
            timeout,
            start: Instant::now(),
            throttle,
        }
    }
}

impl WaiterTrait for TimeoutThrottleWaiter {
    fn start(&mut self) {
        self.start = Instant::now();
    }
    fn wait(&self) -> Result<(), AgentError> {
        if self.start.elapsed() > self.timeout {
            Err(AgentError::TimeoutWaitingForResponse)
        } else {
            std::thread::sleep(self.throttle);

            Ok(())
        }
    }
}
