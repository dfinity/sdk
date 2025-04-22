use crate::workflow::execute::execute::Execute;
use crate::workflow::registry::edge::EdgeType;
use crate::workflow::registry::error::NodeConstructorError;
use crate::workflow::registry::node_config::NodeConfig;
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
