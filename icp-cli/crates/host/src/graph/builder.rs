use crate::node::Node;
use crate::output_promise::OutputPromise;
use crate::registry::node_config::NodeConfig;
use crate::registry::node_type_registry::NodeTypeRegistry;
use crate::workflow::Workflow;
use std::collections::HashMap;
use std::sync::Arc;

pub fn build_graph(yaml: Workflow, registry: &NodeTypeRegistry) -> Vec<Arc<dyn Node>> {
    let mut promises: HashMap<String, Arc<OutputPromise>> = HashMap::new();
    let mut nodes = HashMap::new();

    for (name, node_yaml) in yaml.nodes {
        let node_type_name = node_yaml.r#type.clone().unwrap_or_else(|| name.clone());
        let node_type = registry.get(&node_type_name).expect("unknown node type");

        let mut config = NodeConfig {
            params: HashMap::new(),
            inputs: HashMap::new(),
        };

        // fill params (ConstNode specific for now)
        if let Some(value) = node_yaml.value {
            config.params.insert("value".into(), value);
        }

        // fill inputs
        for (input_name, source_node_name) in node_yaml.inputs {
            let output = promises
                .get(&source_node_name)
                .expect("unknown input node")
                .clone();
            config.inputs.insert(input_name, output);
        }

        let node = (node_type.constructor)(config);
        nodes.insert(name.clone(), node.clone());

        for output_name in &node_type.outputs {
            // assume one OutputPromise per node for now
            let output_promise = node.output_promise();
            promises.insert(name.clone(), output_promise);
        }
    }

    nodes.values().cloned().collect()
}
