use crate::node::Node;
use crate::output_promise::OutputPromise;
use crate::registry::node_config::NodeConfig;
use crate::registry::node_type_registry::NodeTypeRegistry;
use crate::workflow::Workflow;
use std::collections::HashMap;
use std::sync::Arc;

pub fn build_graph(wf: Workflow, registry: &NodeTypeRegistry) -> Vec<Arc<dyn Node>> {
    let mut promises: HashMap<String, Arc<OutputPromise>> = HashMap::new();
    let mut graph_nodes = HashMap::new();

    for node in wf.nodes {
        // eprintln!("node name is '{}'", node.name);
        let node_type_name = node.r#type.clone(); // .unwrap_or_else(|| name.clone());
        let node_type = registry.get(&node_type_name).expect("unknown node type");

        let mut config = NodeConfig {
            params: HashMap::new(),
            inputs: HashMap::new(),
        };

        // fill params (ConstNode specific for now)
        if let Some(value) = node.value {
            config.params.insert("value".into(), value);
        }

        // fill inputs
        for (input_name, source_node_name) in node.inputs {
            //eprintln!("source_node_name is '{source_node_name}'");
            let output = promises
                .get(&source_node_name)
                .expect("unknown input node")
                .clone();
            config.inputs.insert(input_name, output);
        }

        let graph_node = (node_type.constructor)(config);
        graph_nodes.insert(node.name.clone(), graph_node.clone());

        for output_name in &node_type.outputs {
            // assume one OutputPromise per node for now
            let output_promise = graph_node.output_promise();
            promises.insert(node.name.clone(), output_promise);
        }
    }

    graph_nodes.values().cloned().collect()
}
