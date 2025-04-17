use crate::plan::workflow::WorkflowPlan;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct WorkflowModel {
    pub workflow: HashMap<String, NodeModel>,
}

#[derive(Deserialize)]
pub struct NodeModel {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub value: Option<String>, // for ConstNode
    #[serde(default)]
    pub inputs: HashMap<String, String>, // input name → source node name
}

impl WorkflowModel {
    pub(crate) fn from_string(s: &str) -> Self {
        serde_yaml::from_str(s).expect("Failed to parse YAML")
    }
    pub fn into_plan(self) -> WorkflowPlan {
        WorkflowPlan::from_model(self)
    }
}
