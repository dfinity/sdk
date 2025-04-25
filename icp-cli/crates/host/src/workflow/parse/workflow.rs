use crate::workflow::plan::workflow::WorkflowPlan;
use crate::workflow::registry::node_type_registry::NodeTypeRegistry;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum ParameterDefinition {
    #[serde(rename = "node-type")]
    NodeType(NodeTypeParam),

    #[serde(rename = "string")]
    String(StringParam),
}

#[derive(Clone, Debug, Deserialize)]
pub struct NodeTypeParam {
    pub target: String,
}

#[derive(Clone, Debug, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_short_form_parameter() {
        let yaml = r#"
parameters:
  rust-package: builder.package
workflow: {}
"#;
        let model = WorkflowModel::from_string(yaml);
        assert!(matches!(
            model.parameters.get("rust-package"),
            Some(ParameterModel::ShortForm(s)) if s == "builder.package"
        ));
    }

    #[test]
    fn parses_node_type_parameter() {
        let yaml = r#"
parameters:
  builder-type:
    kind: node-type
    target: builder
workflow: {}
"#;
        let model = WorkflowModel::from_string(yaml);
        match model.parameters.get("builder-type") {
            Some(ParameterModel::LongForm(ParameterDefinition::NodeType(param))) => {
                assert_eq!(param.target, "builder");
            }
            _ => panic!("unexpected parameter format"),
        }
    }

    #[test]
    fn parses_string_parameter_with_default() {
        let yaml = r#"
parameters:
  custom-name:
    kind: string
    target: my-node
    property: name
    default: "MyNode"
workflow: {}
"#;
        let model = WorkflowModel::from_string(yaml);
        match model.parameters.get("custom-name") {
            Some(ParameterModel::LongForm(ParameterDefinition::String(param))) => {
                assert_eq!(param.target, "my-node");
                assert_eq!(param.property, "name");
                assert_eq!(param.default.as_deref(), Some("MyNode"));
            }
            _ => panic!("unexpected parameter format"),
        }
    }
}
