use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Workflow {
    pub nodes: HashMap<String, WorkflowNode>,
}

#[derive(Deserialize)]
pub struct WorkflowNode {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub value: Option<String>, // for ConstNode
    #[serde(default)]
    pub inputs: HashMap<String, String>, // input name → source node name
}
