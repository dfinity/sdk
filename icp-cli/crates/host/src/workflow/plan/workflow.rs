use crate::workflow::execute::error::ExecutionGraphFromPlanError;
use crate::workflow::execute::ExecutionGraph;
use crate::workflow::parse::workflow::{NodeModel, WorkflowModel};
use crate::workflow::plan::parameters::{NodeParameterBindings, WorkflowParameterBindings};
use crate::workflow::registry::node_type_registry::NodeTypeRegistry;
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

pub struct WorkflowPlan {
    pub nodes: Vec<WorkflowPlanNode>,
}

pub struct WorkflowPlanNode {
    pub name: String,
    pub r#type: String,
    pub properties: HashMap<String, String>, // input name -> value
    pub inputs: HashMap<String, String>,     // input name → source node name
}

impl WorkflowPlanNode {
    fn new(name: String, yaml: NodeModel, node_bindings: Option<&NodeParameterBindings>) -> Self {
        let inputs = yaml
            .inputs
            .iter()
            .map(|(k, v)| {
                let v = if v.contains(".") {
                    v.clone()
                } else {
                    format!("{}.output", v).clone()
                };
                (k.clone(), v)
            })
            .collect();

        let node_type = node_bindings
            .and_then(|p| p.node_type.clone())
            .or(yaml.r#type.clone())
            .unwrap_or(name.clone());

        // Build the final `properties` with parameters injected
        let mut properties = yaml.properties.clone();
        if let Some(parameters) = node_bindings {
            for (input, value) in &parameters.properties {
                properties.insert(input.clone(), value.clone());
            }
        }
        Self {
            name: name.clone(),
            r#type: node_type,
            properties,
            inputs,
        }
    }
}

impl WorkflowPlan {
    pub fn from_model(
        model: WorkflowModel,
        parameter_values: HashMap<String, String>,
        registry: &NodeTypeRegistry,
    ) -> Self {
        let parameter_bindings =
            WorkflowParameterBindings::from_model(&model, parameter_values, registry);
        let nodes = model
            .workflow
            .into_iter()
            .map(|(name, node)| {
                WorkflowPlanNode::new(name.clone(), node, parameter_bindings.get_node(&name))
            })
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
    let mut sorted = vec![];
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    let mut node_map: HashMap<String, WorkflowPlanNode> = HashMap::new();

    // Build dependency map and lookup map
    for node in nodes {
        let input_names = node
            .inputs
            .values()
            .map(|fqn| fqn.split('.').next().unwrap().to_string())
            .collect();
        deps.insert(node.name.clone(), input_names);
        node_map.insert(node.name.clone(), node);
    }

    // Find starting nodes (no dependencies)
    let mut ready: Vec<String> = deps
        .iter()
        .filter(|(_, inputs)| inputs.is_empty())
        .map(|(name, _)| name.clone())
        .collect();

    while let Some(node_name) = ready.pop() {
        let node = node_map.remove(&node_name).unwrap();
        sorted.push(node);

        // Remove this node as a dependency from others
        for (_other_name, inputs) in deps.iter_mut() {
            inputs.remove(&node_name);
        }

        // Find new ready nodes
        for (name, inputs) in deps.iter() {
            if inputs.is_empty() && !sorted.iter().any(|n| &n.name == name) && !ready.contains(name)
            {
                ready.push(name.clone());
            }
        }
    }

    // If any remain, there's a cycle
    if !node_map.is_empty() {
        let remaining: Vec<_> = node_map.keys().cloned().collect();
        return Err(TopoSortError::CycleDetected(remaining.join(", ")));
    }

    Ok(sorted)
}

pub fn topological_sort_dfs(
    nodes: Vec<WorkflowPlanNode>,
) -> Result<Vec<WorkflowPlanNode>, TopoSortError> {
    let mut sorted = Vec::new();
    let mut deps: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    let mut node_map: HashMap<String, WorkflowPlanNode> = HashMap::new();

    for node in nodes {
        let input_names: HashSet<String> = node.inputs.values().cloned().collect();
        deps.insert(node.name.clone(), input_names);
        node_map.insert(node.name.clone(), node);
    }

    #[derive(PartialEq)]
    enum VisitState {
        Unvisited,
        Visiting,
        Visited,
    }

    let mut visit_state: HashMap<String, VisitState> = deps
        .keys()
        .map(|name| (name.clone(), VisitState::Unvisited))
        .collect();

    let mut stack = Vec::new();

    fn visit(
        name: &str,
        deps: &BTreeMap<String, HashSet<String>>,
        visit_state: &mut HashMap<String, VisitState>,
        stack: &mut Vec<String>,
        sorted: &mut Vec<String>,
    ) -> Result<(), TopoSortError> {
        match visit_state.get(name) {
            Some(VisitState::Visited) => return Ok(()),
            Some(VisitState::Visiting) => {
                // Found a cycle!
                let start = stack.iter().position(|n| n == name).unwrap();
                let cycle: Vec<String> = stack[start..]
                    .iter()
                    .cloned()
                    .chain(std::iter::once(name.to_string()))
                    .collect();
                return Err(TopoSortError::CycleDetected(cycle.join(" → ")));
            }
            _ => {}
        }

        visit_state.insert(name.to_string(), VisitState::Visiting);
        stack.push(name.to_string());

        if let Some(inputs) = deps.get(name) {
            for input in inputs {
                visit(input, deps, visit_state, stack, sorted)?;
            }
        }

        visit_state.insert(name.to_string(), VisitState::Visited);
        sorted.push(name.to_string());
        stack.pop();

        Ok(())
    }

    for name in deps.keys() {
        if visit_state[name] == VisitState::Unvisited {
            visit(name, &deps, &mut visit_state, &mut stack, &mut sorted)?;
        }
    }

    // Build final sorted WorkflowNode list
    let sorted_nodes = sorted
        .into_iter()
        .filter_map(|name| node_map.remove(&name))
        .collect();

    Ok(sorted_nodes)
}

#[cfg(test)]
mod topological_sort_dfs_tests {
    use crate::workflow::plan::workflow::{topological_sort_dfs, TopoSortError, WorkflowPlanNode};
    use std::collections::HashMap;

    #[test]
    fn detects_named_cycle() {
        let nodes = vec![
            WorkflowPlanNode {
                name: "a".to_string(),
                r#type: "const".to_string(),
                properties: HashMap::new(),
                inputs: HashMap::from([("x".into(), "c".into())]),
            },
            WorkflowPlanNode {
                name: "b".to_string(),
                r#type: "print".to_string(),
                properties: HashMap::new(),
                inputs: HashMap::from([("value".into(), "a".into())]),
            },
            WorkflowPlanNode {
                name: "c".to_string(),
                r#type: "print".to_string(),
                properties: HashMap::new(),
                inputs: HashMap::from([("value".into(), "b".into())]),
            },
        ];

        let result = topological_sort_dfs(nodes);
        assert!(matches!(result, Err(TopoSortError::CycleDetected(_))));

        if let Err(TopoSortError::CycleDetected(msg)) = result {
            println!("Cycle: {}", msg);
            assert!(msg.contains("a → c → b → a"));
        }
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
      dummy: b
  b:
    type: const
    inputs:
      dummy: a
"#;

        let _workflow = WorkflowPlan::from_string(yaml);
    }

    #[test]
    fn sorts_simple_workflow() {
        let yaml = r#"
workflow:
  const:
    value: Hello
  print:
    inputs:
      input: const
"#;

        let workflow = WorkflowPlan::from_string(yaml);
        let order: Vec<_> = workflow.nodes.iter().map(|n| n.name.as_str()).collect();

        // const must come before print
        assert_eq!(order, vec!["const", "print"]);
    }

    #[test]
    fn short_form_parameter() {
        let yaml = r#"
parameters:
  rust-package: const-string.value
workflow:
  const-string:
  print:
    inputs:
      input: const-string
"#;

        let model = WorkflowModel::from_string(yaml);
        let parameter_values: HashMap<String, String> =
            HashMap::from([("rust-package".to_string(), "my-package".to_string())]);
        let registry = NodeTypeRegistry::default(); // Assuming this exists

        let plan = model.into_plan(parameter_values, &registry);

        assert_eq!(plan.nodes.len(), 2);

        // Test the const-node
        let const_node = &plan.nodes[0];
        assert_eq!(const_node.name, "const-string");
        assert_eq!(const_node.r#type, "const-string");
        assert_eq!(const_node.properties["value"], "my-package");

        // Test the print-node
        let print_node = &plan.nodes[1];
        assert_eq!(print_node.name, "print");
        assert_eq!(print_node.r#type, "print");
        assert_eq!(print_node.inputs["input"], "const-string.output");
    }

    #[test]
    fn node_type_parameter() {
        let yaml = r#"
parameters:
  transformer-type:
    kind: node-type
    target: transformer
workflow:
  const-string:
    properties:
      value: "some-value"
  transformer:
    inputs:
      input: const-string
  print:
    inputs:
      input: transformer
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
        assert_eq!(print_node.inputs["input"], "transformer.output");
    }
}
