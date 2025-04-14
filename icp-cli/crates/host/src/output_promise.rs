use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

pub struct OutputPromise<T: Clone + Send + Sync + 'static> {
    value: Mutex<Option<T>>,
    notify: Notify,
}

impl<T: Clone + Send + Sync + 'static> OutputPromise<T> {
    pub fn new() -> Self {
        Self {
            value: Mutex::new(None),
            notify: Notify::new(),
        }
    }

    pub async fn get(&self) -> T {
        loop {
            if let Some(val) = self.value.lock().await.clone() {
                return val;
            }
            self.notify.notified().await;
        }
    }

    pub async fn set(&self, val: T) {
        let mut lock = self.value.lock().await;
        *lock = Some(val);
        self.notify.notify_waiters();
    }
}
