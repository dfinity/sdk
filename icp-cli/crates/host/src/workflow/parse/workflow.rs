use crate::workflow::plan::workflow::WorkflowPlan;
use crate::workflow::registry::node_type_registry::NodeTypeRegistry;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum ParameterDefinition {
    #[serde(rename = "string")]
    String(StringParam),
}

#[derive(Clone, Debug, Deserialize)]
pub struct StringParam {
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Deserialize)]
pub struct WorkflowModel {
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,

    pub workflow: HashMap<String, NodeModel>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum InputBinding {
    Literal(String),
    Parameter { parameter: String },
    Node { node: String },
}

#[derive(Deserialize)]
pub struct NodeModel {
    #[serde(default)]
    pub r#type: Option<String>,

    #[serde(default)]
    pub inputs: HashMap<String, InputBinding>,
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
  rust-package:
    kind: string
workflow: {}
"#;
        let model = WorkflowModel::from_string(yaml);
        assert!(matches!(
            model.parameters.get("rust-package"),
            Some(ParameterDefinition::String(_))
        ));
    }

    #[test]
    fn parses_string_parameter_with_default() {
        let yaml = r#"
parameters:
  custom-name:
    kind: string
    default: "MyNode"
workflow: {}
"#;
        let model = WorkflowModel::from_string(yaml);
        match model.parameters.get("custom-name") {
            Some(ParameterDefinition::String(param)) => {
                assert_eq!(param.default.as_deref(), Some("MyNode"));
            }
            _ => panic!("unexpected parameter format"),
        }
    }
}
