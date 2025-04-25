use crate::workflow::parse::workflow::WorkflowModel;
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

impl WorkflowParameterBindings {
    pub fn from_model(
        model: &WorkflowModel,
        parameter_values: HashMap<String, String>,
        registry: &NodeTypeRegistry,
    ) -> Self {
        let mut bindings: HashMap<NodeName, NodeParameterBindings> = HashMap::new();

        // for (param, target) in model.parameters {
        //     let (node, input) = split_target(target)?;
        //     let node_props = &model.workflow[&node].properties;
        //     if node_props.contains_key(&input) {
        //         return Err(anyhow!(
        //         "Parameter '{}' conflicts with existing property '{}.{}'",
        //         param, node, input
        //     ));
        //     }
        //
        //     bindings.entry(node)
        //         .or_default()
        //         .insert(input, param.clone());
        // }

        Self { bindings }
    }

    pub fn get_node(&self, node: &str) -> Option<&NodeParameterBindings> {
        self.bindings.get(node)
    }
}
