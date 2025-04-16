#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::build_graph;
    use crate::nodes::node_descriptors;
    use crate::registry::node_type_registry::NodeTypeRegistry;
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
            r.register(node_descriptors());
            r
        };

        let workflow: Workflow = Workflow::from_string(SIMPLE_WORKFLOW_YAML);

        let graph = build_graph(workflow, &registry);

        let r = graph.run_future.await;
        assert!(r.is_ok(), "Workflow execution failed: {:?}", r);
    }
}
