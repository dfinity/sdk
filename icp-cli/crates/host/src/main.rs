mod bindings;
mod command_config;
mod execution;
mod host;
mod nodes;
mod prettify;
mod registry;
mod tests;
mod workflow;

use crate::nodes::node_descriptors;
use crate::registry::node_type_registry::NodeTypeRegistry;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let workflow = r#"
workflow:
    const:
        value: Hello, test!
    prettify:
        type: prettify
        inputs:
            input: const
    print:
        inputs:
            input: prettify
    print2:
        type: print
        inputs:
            input: const
"#;
    let workflow: workflow::Workflow = workflow::Workflow::from_string(workflow);

    let mut registry = NodeTypeRegistry::new();
    registry.register(node_descriptors());
    let graph = execution::build_graph(workflow, &registry);
    let result = graph.run_future.await;
    if let Err(e) = result {
        println!("Error executing workflow: {}", e);
        std::process::exit(1);
    }
}
