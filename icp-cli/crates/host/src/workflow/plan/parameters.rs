use crate::workflow::parse::workflow::{
    ParameterDefinition, ParameterModel, StringParam, WorkflowModel,
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

impl WorkflowParameterBindings {
    pub fn from_model(
        model: &WorkflowModel,
        parameter_values: HashMap<String, String>,
        registry: &NodeTypeRegistry,
    ) -> Self {
        let mut bindings: HashMap<NodeName, NodeParameterBindings> = HashMap::new();

        for (parameter_name, parameter_model) in &model.parameters {
            let parameter_definition = Self::get_parameter_definition(parameter_model);
            match parameter_definition {
                ParameterDefinition::NodeType(node_type) => {
                    let node_name = node_type.target;
                    if !model.workflow.contains_key(&node_name) {
                        panic!("Node '{}' not found in workflow", node_name);
                    }
                    let Some(parameter_value) = parameter_values.get(parameter_name) else {
                        panic!(
                            "Parameter '{}' not found in parameter values",
                            parameter_name
                        );
                    };
                    if registry.get(parameter_value).is_none() {
                        panic!("Node type '{}' not found in registry", parameter_value);
                    }
                    let node_bindings = bindings.entry(node_name.clone()).or_insert_with(|| {
                        NodeParameterBindings {
                            node_type: None,
                            properties: HashMap::new(),
                        }
                    });
                    if node_bindings.node_type.is_some() {
                        panic!("Node '{}' already has a type defined", node_name);
                    }
                    node_bindings.node_type = Some(parameter_value.clone());
                }
                ParameterDefinition::String(string_param) => {
                    let target = &string_param.target;
                    if !model.workflow.contains_key(target) {
                        panic!("Node '{}' not found in workflow", target);
                    }

                    let Some(parameter_value) = parameter_values
                        .get(parameter_name)
                        .or(string_param.default.as_ref())
                    else {
                        panic!(
                            "Parameter '{}' not found in parameter values",
                            parameter_name
                        );
                    };
                    let Some(node) = model.workflow.get(target) else {
                        panic!("Node '{}' not found in workflow", target);
                    };
                    let node_type = node.r#type.clone().unwrap_or_else(|| target.clone());
                    let Some(descriptor) = registry.get(&node_type) else {
                        panic!("Node type '{}' not found in registry", node_type);
                    };
                    let Some(input) = descriptor.inputs.get(&string_param.property) else {
                        panic!(
                            "Input '{}' not found in node type '{}'",
                            string_param.property, node_type
                        );
                    };
                    if !matches!(input, EdgeType::String) {
                        panic!(
                            "Input '{}' is not of type 'string' in node type '{}'",
                            string_param.property, node_type
                        );
                    }

                    let node_bindings =
                        bindings
                            .entry(target.clone())
                            .or_insert_with(|| NodeParameterBindings {
                                node_type: None,
                                properties: HashMap::new(),
                            });
                    if node_bindings
                        .properties
                        .contains_key(&string_param.property)
                    {
                        panic!(
                            "Property '{}' already defined for node '{}'",
                            &string_param.property, target
                        );
                    }
                    node_bindings
                        .properties
                        .insert(string_param.property.clone(), parameter_value.clone());
                }
            }
        }

        Self { bindings }
    }

    pub fn get_parameter_definition(parameter_model: &ParameterModel) -> ParameterDefinition {
        match parameter_model {
            ParameterModel::ShortForm(target) => {
                let parts: Vec<&str> = target.split('.').collect();
                if parts.len() != 2 {
                    panic!("Invalid parameter definition: {}", target);
                }
                let target = parts[0].to_string();
                let property = parts[1].to_string();
                // todo: check property exists and match type
                ParameterDefinition::String(StringParam {
                    target,
                    property,
                    default: None,
                })
            }
            ParameterModel::LongForm(definition) => definition.clone(),
        }
    }

    pub fn get_node(&self, node: &str) -> Option<&NodeParameterBindings> {
        self.bindings.get(node)
    }
}
