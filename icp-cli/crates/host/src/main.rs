mod bindings;
mod command_config;
mod execute;
mod host;
mod nodes;
mod parse;
mod plan;
mod prettify;
mod registry;
mod tests;

use crate::nodes::node_descriptors;
use crate::parse::workflow::WorkflowModel;
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
    let mut registry = NodeTypeRegistry::new();
    registry.register(node_descriptors());

    // let model = WorkflowModel::from_string(workflow);
    // let plan = WorkflowPlan::from_model(model);
    // let graph = ExecutionGraph::from_plan(plan, &registry);

    let graph = WorkflowModel::from_string(workflow)
        .into_plan()
        .into_graph(&registry);

    let result = graph.run().await;
    if let Err(e) = result {
        println!("Error executing workflow: {}", e);
        std::process::exit(1);
    }
}
