use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct WorkflowYaml {
    pub workflow: HashMap<String, NodeYaml>,
}

#[derive(Deserialize)]
pub struct NodeYaml {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub value: Option<String>, // for ConstNode
    #[serde(default)]
    pub inputs: HashMap<String, String>, // input name → source node name
}
