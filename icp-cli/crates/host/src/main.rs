mod bindings;
mod command_config;
mod graph;
mod host;
mod nodes;
mod output_promise;
mod prettify;
mod registry;
mod tests;
mod workflow;

use crate::nodes::node_descriptors;
use crate::registry::node_type_registry::NodeTypeRegistry;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let workflow = r#"
nodes:
    const:
        value: Hello, test!
    print1:
        type: print
        inputs:
            input: prettify.output
    prettify:
        type: prettify
        inputs:
            input: const.output
    print2:
        type: print
        inputs:
            input: const.output
"#;
    let workflow: workflow::Workflow = workflow::Workflow::from_string(workflow);

    let mut registry = NodeTypeRegistry::new();
    registry.register(node_descriptors());
    let graph = graph::build_graph(workflow, &registry);
    graph.run_future.await
}
