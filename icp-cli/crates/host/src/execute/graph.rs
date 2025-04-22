use crate::execute::error::ExecutionGraphFromPlanError;
use crate::execute::execute::{Execute, SharedExecuteResult};
use crate::execute::promise::{AnyPromise, ExecuteHandle, Promise};
use crate::plan::workflow::WorkflowPlan;
use crate::registry::edge::EdgeType;
use crate::registry::node_config::NodeConfig;
use crate::registry::node_type_registry::NodeTypeRegistry;
use futures_util::future::BoxFuture;
use futures_util::future::FutureExt;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ExecutionGraph {
    pub nodes: Vec<Arc<dyn Execute>>,
    pub run_future: BoxFuture<'static, SharedExecuteResult>,
}

impl ExecutionGraph {
    pub async fn run(self) -> SharedExecuteResult {
        self.run_future.await?;
        Ok(())
    }

    pub fn from_plan(
        wf: WorkflowPlan,
        registry: &NodeTypeRegistry,
    ) -> Result<ExecutionGraph, ExecutionGraphFromPlanError> {
        let mut promises: HashMap<String, AnyPromise> = HashMap::new();
        let mut graph_nodes = HashMap::new();
        let mut side_effect_futures = vec![];

        for node in wf.nodes {
            let node_type_name = node.r#type.clone();
            let node_type = registry.get(&node_type_name).expect("unknown node type");

            let mut config = NodeConfig {
                params: HashMap::new(),
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            };

            // fill params
            if let Some(value) = node.value {
                config.params.insert("value".into(), value);
            }

            // fill inputs
            for (input_name, source_name) in node.inputs {
                let input = promises
                    .get(&source_name)
                    .expect("unknown input node")
                    .clone();
                config.inputs.insert(input_name, input);
            }

            let execute_handle = ExecuteHandle::new();

            // create and register this node's output promises
            for (output_name, edge_type) in &node_type.outputs {
                let fq_name = format!("{}.{}", node.name, output_name);
                let promise = match edge_type {
                    EdgeType::String => {
                        AnyPromise::String(Arc::new(Promise::new(execute_handle.clone())))
                    }
                    EdgeType::Wasm => {
                        AnyPromise::Wasm(Arc::new(Promise::new(execute_handle.clone())))
                    }
                };
                config.outputs.insert(output_name.clone(), promise.clone());
                promises.insert(fq_name, promise);
            }

            // construct node with config
            let graph_node = (node_type.constructor)(config)?;

            // 4. Build eval future
            let execute_future = graph_node.clone().execute().boxed().shared();
            execute_handle.set_execute_future(execute_future.clone());

            // if descriptor has side effect, add to futures
            if node_type.produces_side_effect {
                side_effect_futures.push(execute_future);
            }
            graph_nodes.insert(node.name.clone(), graph_node.clone());
        }

        let nodes = graph_nodes.values().cloned().collect();
        let run_future = futures::future::join_all(side_effect_futures)
            .map(|results| {
                results
                    .into_iter()
                    .find_map(|result| result.err())
                    .map_or(Ok(()), Err)
            })
            .boxed();

        Ok(ExecutionGraph { nodes, run_future })
    }
}
