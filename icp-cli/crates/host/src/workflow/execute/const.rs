use crate::workflow::execute::promise::Input;
use crate::workflow::execute::GraphExecutionError;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ConstPromise<T: Clone + Send + 'static + std::fmt::Debug> {
    value: T,
}

impl<T: Clone + Send + 'static + 'static + std::fmt::Debug> ConstPromise<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static + std::fmt::Debug> Input<T> for ConstPromise<T> {
    async fn get(&self) -> Result<T, Arc<GraphExecutionError>> {
        Ok(self.value.clone())
    }
}
