use crate::execute::execute::Execute;
use crate::registry::node_config::NodeConfig;
use std::sync::Arc;

pub struct NodeDescriptor {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub produces_side_effect: bool,
    pub constructor: Box<dyn Fn(NodeConfig) -> Arc<dyn Execute> + Send + Sync>,
}
