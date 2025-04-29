use crate::workflow::parse::workflow::{
    NodeModel, ParameterDefinition, ParameterModel, StringParam, WorkflowModel,
};
use crate::workflow::registry::edge::EdgeType;
use crate::workflow::registry::node_type_registry::NodeTypeRegistry;
use std::collections::HashMap;

type PropertyName = String;
type NodeName = String;

pub(crate) struct NodeParameterBindings {
    pub(crate) node_type: Option<String>,
    pub(crate) properties: HashMap<PropertyName, String>,
}

pub struct WorkflowParameterBindings {
    bindings: HashMap<NodeName, NodeParameterBindings>,
}

impl WorkflowParameterBindings {}
