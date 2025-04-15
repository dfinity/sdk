use crate::node::Node;
use crate::registry::node_config::NodeConfig;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeType {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constructor: fn(NodeConfig) -> Arc<dyn Node>,
}
