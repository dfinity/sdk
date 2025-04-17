use crate::execution::execute::{Execute, SharedExecuteResult};
use futures_util::future::BoxFuture;
use std::sync::Arc;

pub struct ExecutionGraph {
    pub nodes: Vec<Arc<dyn Execute>>,
    pub run_future: BoxFuture<'static, SharedExecuteResult>,
}
