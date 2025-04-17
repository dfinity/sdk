use crate::execute::error::GraphExecutionError;
use std::sync::Arc;

pub type SharedExecuteResult = Result<(), Arc<GraphExecutionError>>;

#[async_trait::async_trait]
pub trait Execute: Send + Sync
where
    Self: 'static,
{
    async fn execute(self: Arc<Self>) -> SharedExecuteResult;
}
