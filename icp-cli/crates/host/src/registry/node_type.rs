use crate::execute::execute::Execute;
use crate::registry::edge::EdgeType;
use crate::registry::error::NodeConstructorError;
use crate::registry::node_config::NodeConfig;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeDescriptor {
    pub name: String,
    pub inputs: HashMap<String, EdgeType>,
    pub outputs: HashMap<String, EdgeType>,
    pub produces_side_effect: bool,
    pub constructor:
        Box<dyn Fn(NodeConfig) -> Result<Arc<dyn Execute>, NodeConstructorError> + Send + Sync>,
}
