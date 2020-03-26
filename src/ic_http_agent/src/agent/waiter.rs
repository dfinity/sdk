use crate::agent::agent_error::AgentError;
use std::cell::RefCell;
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
    pub fn exponential_backoff(initial: Duration, delta: Duration, multiplier: f32) -> Self {
        Self::from(Box::new(ExponentialBackoffWaiter::new(
            initial, delta, multiplier,
        )))
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
    pub fn exponential_backoff(self, initial: Duration, delta: Duration, multiplier: f32) -> Self {
        self.with(Waiter::exponential_backoff(initial, delta, multiplier))
    }
    pub fn build(mut self) -> Waiter {
        self.inner.take().unwrap_or_else(Waiter::instant)
    }
}

struct ComposeWaiter {
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

struct InstantWaiter {}
impl WaiterTrait for InstantWaiter {
    fn wait(&self) -> Result<(), AgentError> {
        Ok(())
    }
}

struct TimeoutWaiter {
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

struct ThrottleWaiter {
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

struct ExponentialBackoffWaiter {
    next: RefCell<Duration>,
    initial: Duration,
    delta: Duration,
    multiplier: f32,
}

impl ExponentialBackoffWaiter {
    pub fn new(initial: Duration, delta: Duration, multiplier: f32) -> Self {
        ExponentialBackoffWaiter {
            next: RefCell::new(initial),
            initial,
            delta,
            multiplier,
        }
    }
}

impl WaiterTrait for ExponentialBackoffWaiter {
    fn start(&mut self) {
        self.next = RefCell::new(self.initial);
    }

    fn wait(&self) -> Result<(), AgentError> {
        let current = *self.next.borrow();
        // Find the next throttle.
        self.next
            .replace(current.mul_f32(self.multiplier) + self.delta);

        std::thread::sleep(current);

        Ok(())
    }
}
