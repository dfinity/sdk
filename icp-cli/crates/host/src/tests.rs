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
      input: const.output
"#;

    #[tokio::test]
    async fn test_simple_workflow_builds_and_runs() {
        let registry = {
            let mut r = NodeTypeRegistry::new();
            r.register(node_types());
            r
        };

        let workflow: Workflow = Workflow::from_string(SIMPLE_WORKFLOW_YAML);

        let graph = build_graph(workflow, &registry);

        graph.run_future.await

        // no asserts needed yet — just validating no panics, visible logs
    }
}
