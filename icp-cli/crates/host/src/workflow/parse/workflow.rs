use crate::workflow::plan::workflow::WorkflowPlan;
use crate::workflow::registry::node_type_registry::NodeTypeRegistry;
use serde::Deserialize;
use std::collections::HashMap;
/*
parameters:
  rust-package: builder.package  # short form
  builder-type:                  # long form to for a node-type parameter
    kind: node-type
    target: builder
  custom-name:                   # long-form to set a string property parameter
    kind: string
    target: my-node
    property: name
    default: "MyNode"


 */

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum ParameterDefinition {
    #[serde(rename = "node-type")]
    NodeType(NodeTypeParam),

    #[serde(rename = "string")]
    String(StringParam),
}

#[derive(Debug, Deserialize)]
pub struct NodeTypeParam {
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub struct StringParam {
    pub target: String,
    pub property: String,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParameterModel {
    ShortForm(String), // e.g. "builder.package"
    LongForm(ParameterDefinition),
}

#[derive(Deserialize)]
pub struct WorkflowModel {
    #[serde(default)]
    pub parameters: HashMap<String, ParameterModel>,

    pub workflow: HashMap<String, NodeModel>,
}

#[derive(Deserialize)]
pub struct NodeModel {
    #[serde(default)]
    pub r#type: Option<String>,

    #[serde(default)]
    pub properties: HashMap<String, String>, // node properties

    #[serde(default)]
    pub inputs: HashMap<String, String>, // input name → source node name
}

impl WorkflowModel {
    pub(crate) fn from_string(s: &str) -> Self {
        serde_yaml::from_str(s).expect("Failed to parse YAML")
    }
    pub fn into_plan(
        self,
        parameter_values: HashMap<String, String>,
        registry: &NodeTypeRegistry,
    ) -> WorkflowPlan {
        WorkflowPlan::from_model(self, parameter_values, registry)
    }
}
