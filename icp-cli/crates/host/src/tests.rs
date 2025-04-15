#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::build_graph;
    use crate::nodes::node_types;
    use crate::registry::node_type_registry::NodeTypeRegistry;
    use crate::runtime::Runtime;
    use crate::workflow::Workflow;
    use serde_yaml;

    const SIMPLE_WORKFLOW_YAML: &str = r#"
nodes:
  const:
    value: Hello, test!
  print:
    inputs:
      input: const
"#;

    #[tokio::test]
    async fn test_simple_workflow_builds_and_runs() {
        let registry = {
            let mut r = NodeTypeRegistry::new();
            r.register(node_types());
            r
        };

        let workflow: Workflow =
            serde_yaml::from_str(SIMPLE_WORKFLOW_YAML).expect("failed to parse YAML");

        let nodes = build_graph(workflow, &registry);

        let mut runtime = Runtime::new();
        for node in nodes {
            runtime.add_node(node);
        }
        runtime.run_graph().await;

        // no asserts needed yet — just validating no panics, visible logs
    }
}
