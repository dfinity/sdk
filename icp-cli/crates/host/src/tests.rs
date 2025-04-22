#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::execute::Execute;
    use crate::execute::ExecutionGraph;
    use crate::nodes::node_descriptors;
    use crate::plan::workflow::WorkflowPlan;
    use crate::registry::node_type_registry::NodeTypeRegistry;
    use serde_yaml;

    const SIMPLE_WORKFLOW_YAML: &str = r#"
workflow:
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

        let workflow: WorkflowPlan = WorkflowPlan::from_string(SIMPLE_WORKFLOW_YAML);

        let graph = ExecutionGraph::from_plan(workflow, &registry);

        let r = graph.run_future.await;
        assert!(r.is_ok(), "Workflow execution failed: {:?}", r);
    }
}

#[cfg(test)]
mod lazy_evaluation_test {
    use crate::execute::execute::{Execute, SharedExecuteResult};
    use crate::execute::promise::{Input, InputRef, Output, OutputRef};
    use crate::execute::ExecutionGraph;
    use crate::nodes::edge::EdgeType;
    use crate::plan::workflow::WorkflowPlan;
    use crate::registry::node_config::NodeConfig;
    use crate::registry::node_type::NodeDescriptor;
    use crate::registry::node_type_registry::NodeTypeRegistry;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    // Define a helper function for constructing ConstNode
    fn create_const_node(
        config: &NodeConfig,
        ab: Arc<AtomicBool>,
        log: Arc<Mutex<Vec<String>>>,
    ) -> Arc<dyn Execute> {
        let value = config.string_param("value");
        let output = config.string_output("output");
        Arc::new(LazyNode {
            ab,
            value,
            output,
            log,
        })
    }

    // Define a helper function for constructing PrintNode
    fn create_print_node(
        config: &NodeConfig,
        ab: Arc<AtomicBool>,
        log: Arc<Mutex<Vec<String>>>,
    ) -> Arc<dyn Execute> {
        let input = config.string_source("input");
        Arc::new(EagerNode { ab, input, log })
    }

    pub struct LazyNode {
        ab: Arc<AtomicBool>,
        value: String,
        output: OutputRef<String>,
        log: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Execute for LazyNode {
        async fn execute(self: Arc<Self>) -> SharedExecuteResult {
            // Should be true before executing, then set to false
            self.ab
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                .expect("AtomicBool should have been true before LazyNode execute");

            // Log execute order
            self.log
                .lock()
                .unwrap()
                .push("LazyNode executing".to_string());

            eprintln!("LazyNode executed with value: {:?}", self.value);

            self.output.set(self.value.clone());
            Ok(())
        }
    }

    impl LazyNode {
        pub fn descriptor(ab: Arc<AtomicBool>, log: Arc<Mutex<Vec<String>>>) -> NodeDescriptor {
            let ab_clone = ab.clone();
            let log_clone = log.clone();
            NodeDescriptor {
                name: "lazy".to_string(),
                inputs: HashMap::new(),
                outputs: HashMap::from([("output".to_string(), EdgeType::String)]),
                produces_side_effect: false,
                // Use the helper function here
                constructor: Box::new(move |config| {
                    create_const_node(&config, ab.clone(), log.clone())
                }),
            }
        }
    }

    pub struct EagerNode {
        ab: Arc<AtomicBool>,
        input: InputRef<String>,
        log: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Execute for EagerNode {
        async fn execute(self: Arc<Self>) -> SharedExecuteResult {
            tokio::task::yield_now().await;
            // Should be false before executing, then set to true
            self.ab
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .expect("AtomicBool should have been false before EagerNode execute");

            // Log execute order
            self.log
                .lock()
                .unwrap()
                .push("EagerNode executing".to_string());

            eprintln!("EagerNode executing");
            let value = self.input.get().await?;
            println!("EagerNode received: {value}");
            Ok(())
        }
    }

    impl EagerNode {
        pub fn descriptor(ab: Arc<AtomicBool>, log: Arc<Mutex<Vec<String>>>) -> NodeDescriptor {
            let ab_clone = ab.clone();
            let log_clone = log.clone();

            NodeDescriptor {
                name: "eager".to_string(),
                inputs: HashMap::from([("output".to_string(), EdgeType::String)]),
                outputs: HashMap::new(),
                produces_side_effect: true,
                constructor: Box::new(move |config| {
                    create_print_node(&config, ab_clone.clone(), log_clone.clone())
                }),
            }
        }
    }

    //#[tokio::test(flavor = "current_thread")]
    #[tokio::test(flavor = "multi_thread")]
    async fn lazy_evaluation() {
        let ab = Arc::new(AtomicBool::new(false));

        // Shared log to track execute order
        let log = Arc::new(Mutex::new(Vec::new()));

        let registry = {
            let mut r = NodeTypeRegistry::new();
            r.register(vec![
                LazyNode::descriptor(ab.clone(), log.clone()),
                EagerNode::descriptor(ab.clone(), log.clone()),
            ]);
            r
        };

        const SIMPLE_WORKFLOW_YAML: &str = r#"
workflow:
  lazy:
    value: lazily executed
  eager:
    inputs:
      input: lazy
"#;

        let workflow: WorkflowPlan = WorkflowPlan::from_string(SIMPLE_WORKFLOW_YAML);

        let graph = ExecutionGraph::from_plan(workflow, &registry);
        let r = graph.run_future.await;

        assert!(r.is_ok(), "Workflow execution failed: {:?}", r);

        // Verify execute order by checking log
        let log = log.lock().unwrap();
        assert_eq!(
            *log,
            vec![
                "EagerNode executing".to_string(),
                "LazyNode executing".to_string()
            ]
        );
    }
}
