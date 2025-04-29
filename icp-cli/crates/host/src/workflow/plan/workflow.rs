use crate::workflow::execute::error::ExecutionGraphFromPlanError;
use crate::workflow::execute::ExecutionGraph;
use crate::workflow::parse::workflow::{InputBinding, NodeModel, NodeTypeBinding, WorkflowModel};
use crate::workflow::registry::node_type_registry::NodeTypeRegistry;
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

pub struct WorkflowPlan {
    pub nodes: Vec<WorkflowPlanNode>,
}

pub enum WorkflowInputBinding {
    String(String),
    NodeOutput { node: String, output: String },
}

pub struct WorkflowPlanNode {
    pub name: String,
    pub r#type: String,
    pub inputs: HashMap<String, WorkflowInputBinding>, // input name → source node name
}

impl WorkflowPlanNode {
    fn new(name: String, yaml: NodeModel, parameter_values: &HashMap<String, String>) -> Self {
        let inputs = yaml
            .inputs
            .iter()
            .map(|(k, v)| {
                let v = match v {
                    InputBinding::Literal(literal) => WorkflowInputBinding::String(literal.clone()),
                    InputBinding::Parameter { parameter } => {
                        let v = parameter_values
                            .get(parameter)
                            .unwrap_or_else(|| panic!("Parameter {} not found", parameter));
                        WorkflowInputBinding::String(v.clone())
                    }
                    InputBinding::Node { node } => {
                        let (node, output) = if node.contains(".") {
                            let parts: Vec<_> = node.split('.').collect();
                            let node = parts[0].to_string();
                            let output = parts[1].to_string();
                            (node, output)
                        } else {
                            (node.clone(), "output".to_string())
                        };

                        WorkflowInputBinding::NodeOutput { node, output }
                    }
                };

                (k.clone(), v)
            })
            .collect();

        let node_type = match yaml.r#type {
            None => name.clone(),
            Some(NodeTypeBinding::Type(t)) => t,
            Some(NodeTypeBinding::Parameter { parameter }) => {
                let v = parameter_values
                    .get(&parameter)
                    .unwrap_or_else(|| panic!("Parameter {} not found", parameter));
                v.clone()
            }
        };

        Self {
            name: name.clone(),
            r#type: node_type,
            inputs,
        }
    }
}

impl WorkflowPlan {
    pub fn from_model(
        model: WorkflowModel,
        parameter_values: HashMap<String, String>,
        _registry: &NodeTypeRegistry, // todo: check parameter types
    ) -> Self {
        let nodes = model
            .workflow
            .into_iter()
            .map(|(name, node)| WorkflowPlanNode::new(name.clone(), node, &parameter_values))
            .collect();
        let nodes =
            topological_sort_kahn(nodes).expect("Failed to sort workflow nodes: cycle detected");
        Self { nodes }
    }

    pub fn into_graph(
        self,
        registry: &NodeTypeRegistry,
    ) -> Result<ExecutionGraph, ExecutionGraphFromPlanError> {
        ExecutionGraph::from_plan(self, registry)
    }

    pub fn from_string(s: &str) -> Self {
        let plan = WorkflowModel::from_string(s);
        Self::from_model(plan, HashMap::new(), &NodeTypeRegistry::new())
    }
}

#[derive(Error, Debug)]
pub enum TopoSortError {
    #[error("Cycle detected: {0}")]
    CycleDetected(String),
}

fn topological_sort_kahn(
    nodes: Vec<WorkflowPlanNode>,
) -> Result<Vec<WorkflowPlanNode>, TopoSortError> {
    let mut sorted = Vec::new();
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    let mut node_map: HashMap<String, WorkflowPlanNode> = HashMap::new();

    // Build dependency map and lookup map
    for node in nodes {
        let input_nodes: HashSet<String> = node
            .inputs
            .values()
            .filter_map(|binding| match binding {
                WorkflowInputBinding::NodeOutput { node, .. } => Some(node.clone()),
                _ => None,
            })
            .collect();

        deps.insert(node.name.clone(), input_nodes);
        node_map.insert(node.name.clone(), node);
    }

    // Find nodes with no dependencies
    let mut ready: Vec<String> = deps
        .iter()
        .filter(|(_, deps)| deps.is_empty())
        .map(|(name, _)| name.clone())
        .collect();

    while let Some(name) = ready.pop() {
        let node = node_map.remove(&name).unwrap();
        sorted.push(node);

        // Remove this node from others' dependencies
        for (_other, dep_set) in deps.iter_mut() {
            dep_set.remove(&name);
        }

        // Add new nodes with no remaining deps
        for (name, dep_set) in deps.iter() {
            if dep_set.is_empty()
                && !sorted.iter().any(|n| &n.name == name)
                && !ready.contains(name)
            {
                ready.push(name.clone());
            }
        }
    }

    if !node_map.is_empty() {
        let remaining: Vec<_> = node_map.keys().cloned().collect();
        Err(TopoSortError::CycleDetected(remaining.join(", ")))
    } else {
        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "cycle detected")]
    fn detects_cycle() {
        let yaml = r#"
workflow:
  a:
    type: const
    inputs:
      dummy:
        node: b
  b:
    type: const
    inputs:
      dummy:
        node: a
"#;

        let _workflow = WorkflowPlan::from_string(yaml);
    }

    #[test]
    fn sorts_simple_workflow() {
        let yaml = r#"
workflow:
  const:
    inputs:
      value: Hello
  print:
    inputs:
      input:
        node: const
"#;

        let workflow = WorkflowPlan::from_string(yaml);
        let order: Vec<_> = workflow.nodes.iter().map(|n| n.name.as_str()).collect();

        // const must come before print
        assert_eq!(order, vec!["const", "print"]);
    }

    //     #[test]
    //     fn short_form_parameter() {
    //         let yaml = r#"
    // parameters:
    //   rust-package: const-string.value
    // workflow:
    //   const-string:
    //   print:
    //     inputs:
    //       input: const-string
    // "#;
    //
    //         let model = WorkflowModel::from_string(yaml);
    //         let parameter_values: HashMap<String, String> =
    //             HashMap::from([("rust-package".to_string(), "my-package".to_string())]);
    //         let registry = NodeTypeRegistry::default(); // Assuming this exists
    //
    //         let plan = model.into_plan(parameter_values, &registry);
    //
    //         assert_eq!(plan.nodes.len(), 2);
    //
    //         // Test the const-node
    //         let const_node = &plan.nodes[0];
    //         assert_eq!(const_node.name, "const-string");
    //         assert_eq!(const_node.r#type, "const-string");
    //         assert_eq!(const_node.properties["value"], "my-package");
    //
    //         // Test the print-node
    //         let print_node = &plan.nodes[1];
    //         assert_eq!(print_node.name, "print");
    //         assert_eq!(print_node.r#type, "print");
    //         assert_eq!(print_node.inputs["input"], "const-string.output");
    //     }

    #[test]
    fn node_type_parameter() {
        let yaml = r#"
    parameters:
      transformer-type:
        kind: node-type
    workflow:
      const-string:
        inputs:
          value: "some-value"
      transformer:
        type:
          parameter: transformer-type
        inputs:
          input:
            node: const-string
      print:
        inputs:
          input:
            node: transformer
    "#;

        let model = WorkflowModel::from_string(yaml);
        let parameter_values: HashMap<String, String> =
            HashMap::from([("transformer-type".to_string(), "prettify".to_string())]);
        let registry = NodeTypeRegistry::default(); // Assuming this exists

        let plan = model.into_plan(parameter_values, &registry);

        assert_eq!(plan.nodes.len(), 3);

        // Test the const-node
        let const_node = &plan.nodes[0];
        assert_eq!(const_node.name, "const-string");
        assert_eq!(const_node.r#type, "const-string");

        // Test the transformer node
        let transformer_node = &plan.nodes[1];
        assert_eq!(transformer_node.name, "transformer");
        assert_eq!(transformer_node.r#type, "prettify");

        // Test the print-node
        let print_node = &plan.nodes[2];
        assert_eq!(print_node.name, "print");
        assert_eq!(print_node.r#type, "print");
        let input_binding = print_node.inputs.get("input").unwrap();
        match input_binding {
            WorkflowInputBinding::NodeOutput { node, output } => {
                assert_eq!(node, "transformer");
                assert_eq!(output, "output");
            }
            _ => panic!("Expected NodeOutput binding"),
        }
    }
}
