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
    pub fn from(inner: Box<dyn WaiterTrait>) -> Self {
        Waiter { inner }
    }

    /// A Waiter that never waits. This can hog resources, so careful.
    pub fn instant() -> Self {
        Self::from(Box::new(InstantWaiter {}))
    }

    pub fn timeout(timeout: Duration) -> Self {
        Self::from(Box::new(TimeoutWaiter::new(timeout)))
    }
    pub fn throttle(throttle: Duration) -> Self {
        Self::from(Box::new(ThrottleWaiter::new(throttle)))
    }
    pub fn builder() -> WaiterBuilder {
        WaiterBuilder { inner: None }
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

pub struct WaiterBuilder {
    inner: Option<Waiter>,
}
impl WaiterBuilder {
    pub fn with(mut self, other: Waiter) -> Self {
        self.inner = Some(match self.inner.take() {
            None => other,
            Some(w) => Waiter::from(Box::new(ComposeWaiter::new(w, other))),
        });
        self
    }
    pub fn timeout(self, timeout: Duration) -> Self {
        self.with(Waiter::timeout(timeout))
    }
    pub fn throttle(self, throttle: Duration) -> Self {
        self.with(Waiter::throttle(throttle))
    }
    pub fn build(mut self) -> Waiter {
        self.inner.take().unwrap_or_else(|| Waiter::instant())
    }
}

pub struct ComposeWaiter {
    a: Waiter,
    b: Waiter,
}
impl ComposeWaiter {
    fn new(a: Waiter, b: Waiter) -> Self {
        Self { a, b }
    }
}
impl WaiterTrait for ComposeWaiter {
    fn start(&mut self) {
        self.a.start();
        self.b.start();
    }
    fn wait(&self) -> Result<(), AgentError> {
        self.a.wait()?;
        self.b.wait()?;
        Ok(())
    }
}

pub struct InstantWaiter {}
impl WaiterTrait for InstantWaiter {
    fn wait(&self) -> Result<(), AgentError> {
        Ok(())
    }
}

pub struct TimeoutWaiter {
    timeout: Duration,
    start: Instant,
}

impl TimeoutWaiter {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            start: Instant::now(),
        }
    }
}

impl WaiterTrait for TimeoutWaiter {
    fn start(&mut self) {
        self.start = Instant::now();
    }
    fn wait(&self) -> Result<(), AgentError> {
        if self.start.elapsed() > self.timeout {
            Err(AgentError::TimeoutWaitingForResponse)
        } else {
            Ok(())
        }
    }
}

pub struct ThrottleWaiter {
    throttle: Duration,
}

impl ThrottleWaiter {
    pub fn new(throttle: Duration) -> Self {
        Self { throttle }
    }
}

impl WaiterTrait for ThrottleWaiter {
    fn wait(&self) -> Result<(), AgentError> {
        std::thread::sleep(self.throttle);

        Ok(())
    }
}
